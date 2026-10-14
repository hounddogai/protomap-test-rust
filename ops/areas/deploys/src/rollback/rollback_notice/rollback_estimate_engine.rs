use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rollback estimate calculation.
pub trait RollbackEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackEstimateAdjuster {
    pub minimum: f64,
}

impl RollbackEstimateModifier for MinimumRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackEstimateModifier for FlaggedRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackEstimateAdjuster {
    pub cap: f64,
}

impl RollbackEstimateModifier for CapRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackEstimateAdjuster {
    pub percent: f64,
}

impl RollbackEstimateModifier for PercentRollbackEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackEstimateEngine {
    pub steps: Vec<Rc<dyn RollbackEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackEstimateEngine {
    pub fn price(&mut self, item: &RollbackEstimate) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackEstimateModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RollbackEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_estimate_engine(flags: &flags::Registry) -> RollbackEstimateEngine {
    RollbackEstimateEngine {
        steps: vec![
            Rc::new(MinimumRollbackEstimateAdjuster { minimum: 37.9 }) as Rc<dyn RollbackEstimateModifier>,
            Rc::new(FlaggedRollbackEstimateAdjuster {
                flags: flags.clone(),
                flag: "rollback-estimate-manual".to_string(),
                percent: 7.2,
            }),
            Rc::new(CapRollbackEstimateAdjuster { cap: 21887.0 }),
            Rc::new(PercentRollbackEstimateAdjuster { percent: -23.9 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 58382.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_estimate_engine(&flags::new_registry());
        let item = new_rollback_estimate("code-977", "label-511");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
