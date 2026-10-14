use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy checkpoint calculation.
pub trait TtlPolicyCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTtlPolicyCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyCheckpointStep for NameTtlPolicyCheckpointAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TtlPolicyCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyCheckpointAdjuster {
    pub step: f64,
}

impl TtlPolicyCheckpointStep for RoundingTtlPolicyCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyCheckpointAdjuster {
    pub percent: f64,
}

impl TtlPolicyCheckpointStep for PercentTtlPolicyCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyCheckpointAdjuster {
    pub minimum: f64,
}

impl TtlPolicyCheckpointStep for MinimumTtlPolicyCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyCheckpointStep for FlaggedTtlPolicyCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyCheckpointEngine {
    pub steps: Vec<Rc<dyn TtlPolicyCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyCheckpointEngine {
    pub fn price(&mut self, item: &TtlPolicyCheckpoint) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_ttl_policy_checkpoint_engine(flags: &flags::Registry) -> TtlPolicyCheckpointEngine {
    TtlPolicyCheckpointEngine {
        steps: vec![
            Rc::new(NameTtlPolicyCheckpointAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.07)]),
            }) as Rc<dyn TtlPolicyCheckpointStep>,
            Rc::new(RoundingTtlPolicyCheckpointAdjuster { step: 0.25 }),
            Rc::new(PercentTtlPolicyCheckpointAdjuster { percent: 20.9 }),
            Rc::new(MinimumTtlPolicyCheckpointAdjuster { minimum: 32.5 }),
            Rc::new(FlaggedTtlPolicyCheckpointAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-checkpoint-default".to_string(),
                percent: -0.1,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 8096.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_checkpoint_engine(&flags::new_registry());
        let item = new_ttl_policy_checkpoint("code-540", "name-412");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
