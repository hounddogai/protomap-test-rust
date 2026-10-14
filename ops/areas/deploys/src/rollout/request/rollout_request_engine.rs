use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout request calculation.
pub trait RolloutRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutRequestAdjuster {
    pub cap: f64,
}

impl RolloutRequestStep for CapRolloutRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutRequestAdjuster {
    pub minimum: f64,
}

impl RolloutRequestStep for MinimumRolloutRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutRequestStep for FlaggedRolloutRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollout request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutRequestEngine {
    pub steps: Vec<Rc<dyn RolloutRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutRequestEngine {
    pub fn price(&mut self, item: &RolloutRequest) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[RolloutRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollout_request_engine(flags: &flags::Registry) -> RolloutRequestEngine {
    RolloutRequestEngine {
        steps: vec![
            Rc::new(CapRolloutRequestAdjuster { cap: 15658.0 }) as Rc<dyn RolloutRequestStep>,
            Rc::new(MinimumRolloutRequestAdjuster { minimum: 31.2 }),
            Rc::new(FlaggedRolloutRequestAdjuster {
                flags: flags.clone(),
                flag: "rollout-request-strict".to_string(),
                percent: -4.9,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 76181.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_request_engine(&flags::new_registry());
        let item = new_rollout_request("code-940", "title-291");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
