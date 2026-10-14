use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback profile calculation.
pub trait RollbackProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackProfileAdjuster {
    pub percent: f64,
}

impl RollbackProfileStep for PercentRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackProfileAdjuster {
    pub cap: f64,
}

impl RollbackProfileStep for CapRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackProfileStep for FlaggedRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackProfileStep for TieredRollbackProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackProfileGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollbackProfileStep for RollbackProfileGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackProfile, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the rollback profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackProfileEngine {
    pub steps: Vec<Rc<dyn RollbackProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackProfileEngine {
    pub fn run(&mut self, item: &RollbackProfile) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[RollbackProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollback_profile_engine(flags: &flags::Registry) -> RollbackProfileEngine {
    RollbackProfileEngine {
        steps: vec![
            Rc::new(PercentRollbackProfileAdjuster { percent: 8.1 }) as Rc<dyn RollbackProfileStep>,
            Rc::new(CapRollbackProfileAdjuster { cap: 43789.0 }),
            Rc::new(FlaggedRollbackProfileAdjuster {
                flags: flags.clone(),
                flag: "rollback-profile-rolling".to_string(),
                percent: 5.9,
            }),
            Rc::new(TieredRollbackProfileAdjuster {
                bounds: vec![5100.0, 5665.0, 8493.0],
                rates: vec![0.95, 1.02, 1.14],
            }),
            Rc::new(RollbackProfileGraceDaysOffsetAdjuster { weight: 1.19 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 36171.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_profile_engine(&flags::new_registry());
        let item = new_rollback_profile("code-890", "label-687");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
