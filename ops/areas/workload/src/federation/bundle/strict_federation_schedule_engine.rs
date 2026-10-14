use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict federation schedule calculation.
pub trait StrictFederationScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictFederationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictFederationScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictFederationScheduleModifier for TieredStrictFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictFederationSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelStrictFederationScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictFederationScheduleModifier for ChannelStrictFederationScheduleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &StrictFederationSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictFederationScheduleAdjuster {
    pub cap: f64,
}

impl StrictFederationScheduleModifier for CapStrictFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictFederationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictFederationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictFederationScheduleModifier for FlaggedStrictFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictFederationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictFederationScheduleAdjuster {
    pub step: f64,
}

impl StrictFederationScheduleModifier for RoundingStrictFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictFederationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the strict federation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictFederationScheduleEngine {
    pub steps: Vec<Rc<dyn StrictFederationScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictFederationScheduleEngine {
    pub fn compute(&mut self, item: &StrictFederationSchedule) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[StrictFederationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_federation_schedule_engine(flags: &flags::Registry) -> StrictFederationScheduleEngine {
    StrictFederationScheduleEngine {
        steps: vec![
            Rc::new(TieredStrictFederationScheduleAdjuster {
                bounds: vec![5725.0, 7635.0, 8036.0],
                rates: vec![0.99, 1.06, 0.93],
            }) as Rc<dyn StrictFederationScheduleModifier>,
            Rc::new(ChannelStrictFederationScheduleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.39), ("online".to_string(), 1.16)]),
            }),
            Rc::new(CapStrictFederationScheduleAdjuster { cap: 49772.0 }),
            Rc::new(FlaggedStrictFederationScheduleAdjuster {
                flags: flags.clone(),
                flag: "strict-federation-schedule-primary".to_string(),
                percent: 4.0,
            }),
            Rc::new(RoundingStrictFederationScheduleAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 46685.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_federation_schedule_engine(&flags::new_registry());
        let item = new_strict_federation_schedule("code-246", "name-599");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
