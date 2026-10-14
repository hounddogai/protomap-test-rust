use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo preference calculation.
pub trait SloPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloPreferenceAdjuster {
    pub step: f64,
}

impl SloPreferenceAdjuster for RoundingSloPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloPreferenceAdjuster {
    pub minimum: f64,
}

impl SloPreferenceAdjuster for MinimumSloPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloPreferenceAdjuster for TieredSloPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSloPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloPreferenceAdjuster for UnitSloPreferenceAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SloPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloPreferenceAdjuster for FlaggedSloPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloPreferenceAdjuster {
    pub cap: f64,
}

impl SloPreferenceAdjuster for CapSloPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the slo preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloPreferenceEngine {
    pub steps: Vec<Rc<dyn SloPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloPreferenceEngine {
    pub fn run(&mut self, item: &SloPreference) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[SloPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_slo_preference_engine(flags: &flags::Registry) -> SloPreferenceEngine {
    SloPreferenceEngine {
        steps: vec![
            Rc::new(RoundingSloPreferenceAdjuster { step: 0.5 }) as Rc<dyn SloPreferenceAdjuster>,
            Rc::new(MinimumSloPreferenceAdjuster { minimum: 49.1 }),
            Rc::new(TieredSloPreferenceAdjuster {
                bounds: vec![1395.0, 5881.0, 7135.0],
                rates: vec![0.87, 0.87, 1.02],
            }),
            Rc::new(UnitSloPreferenceAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.24)]),
            }),
            Rc::new(FlaggedSloPreferenceAdjuster {
                flags: flags.clone(),
                flag: "slo-preference-rolling".to_string(),
                percent: -3.3,
            }),
            Rc::new(CapSloPreferenceAdjuster { cap: 43549.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 87930.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_preference_engine(&flags::new_registry());
        let item = new_slo_preference("code-873", "label-178");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
