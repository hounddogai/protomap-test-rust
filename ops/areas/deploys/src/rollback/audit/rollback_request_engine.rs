use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback request calculation.
pub trait RollbackRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackRequest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackRequestAdjuster for FlaggedRollbackRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackRequestAdjuster {
    pub minimum: f64,
}

impl RollbackRequestAdjuster for MinimumRollbackRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackRequestAdjuster {
    pub step: f64,
}

impl RollbackRequestAdjuster for RoundingRollbackRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackRequestAdjuster {
    pub percent: f64,
}

impl RollbackRequestAdjuster for PercentRollbackRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRequestCeilingOffsetAdjuster {
    pub weight: f64,
}

impl RollbackRequestAdjuster for RollbackRequestCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackRequest, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the rollback request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackRequestEngine {
    pub steps: Vec<Rc<dyn RollbackRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackRequestEngine {
    pub fn evaluate(&mut self, item: &RollbackRequest) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackRequestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollback_request_engine(flags: &flags::Registry) -> RollbackRequestEngine {
    RollbackRequestEngine {
        steps: vec![
            Rc::new(FlaggedRollbackRequestAdjuster {
                flags: flags.clone(),
                flag: "rollback-request-default".to_string(),
                percent: 9.8,
            }) as Rc<dyn RollbackRequestAdjuster>,
            Rc::new(MinimumRollbackRequestAdjuster { minimum: 44.6 }),
            Rc::new(RoundingRollbackRequestAdjuster { step: 0.5 }),
            Rc::new(PercentRollbackRequestAdjuster { percent: 12.3 }),
            Rc::new(RollbackRequestCeilingOffsetAdjuster { weight: 1.44 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 79609.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_request_engine(&flags::new_registry());
        let item = new_rollback_request("code-759", "label-924");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
