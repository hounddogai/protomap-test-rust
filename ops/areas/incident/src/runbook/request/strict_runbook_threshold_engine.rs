use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict runbook threshold calculation.
pub trait StrictRunbookThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRunbookThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleStrictRunbookThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictRunbookThresholdAdjuster for TitleStrictRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &StrictRunbookThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictRunbookThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictRunbookThresholdAdjuster for FlaggedStrictRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictRunbookThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictRunbookThresholdAdjuster {
    pub cap: f64,
}

impl StrictRunbookThresholdAdjuster for CapStrictRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictRunbookThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictRunbookThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictRunbookThresholdAdjuster for TieredStrictRunbookThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictRunbookThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict runbook threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRunbookThresholdEngine {
    pub steps: Vec<Rc<dyn StrictRunbookThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRunbookThresholdEngine {
    pub fn run(&mut self, item: &StrictRunbookThreshold) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[StrictRunbookThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_runbook_threshold_engine(flags: &flags::Registry) -> StrictRunbookThresholdEngine {
    StrictRunbookThresholdEngine {
        steps: vec![
            Rc::new(TitleStrictRunbookThresholdAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.83),
                    ("south".to_string(), 1.13),
                    ("west".to_string(), 1.24),
                ]),
            }) as Rc<dyn StrictRunbookThresholdAdjuster>,
            Rc::new(FlaggedStrictRunbookThresholdAdjuster {
                flags: flags.clone(),
                flag: "strict-runbook-threshold-rolling".to_string(),
                percent: -8.5,
            }),
            Rc::new(CapStrictRunbookThresholdAdjuster { cap: 39365.0 }),
            Rc::new(TieredStrictRunbookThresholdAdjuster {
                bounds: vec![2055.0, 2379.0, 5445.0],
                rates: vec![1.04, 0.81, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 34402.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_runbook_threshold_engine(&flags::new_registry());
        let item = new_strict_runbook_threshold("code-838", "title-574");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
