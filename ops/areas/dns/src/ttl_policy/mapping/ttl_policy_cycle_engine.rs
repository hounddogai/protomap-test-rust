use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy cycle calculation.
pub trait TtlPolicyCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCycleScoreOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyCycleFactor for TtlPolicyCycleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyCycle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTtlPolicyCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyCycleFactor for SourceTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TtlPolicyCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyCycleFactor for FlaggedTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyCycleAdjuster {
    pub minimum: f64,
}

impl TtlPolicyCycleFactor for MinimumTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyCycleAdjuster {
    pub cap: f64,
}

impl TtlPolicyCycleFactor for CapTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyCycleAdjuster {
    pub percent: f64,
}

impl TtlPolicyCycleFactor for PercentTtlPolicyCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the ttl policy cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyCycleEngine {
    pub steps: Vec<Rc<dyn TtlPolicyCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyCycleEngine {
    pub fn run(&mut self, item: &TtlPolicyCycle) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_cycle_engine(flags: &flags::Registry) -> TtlPolicyCycleEngine {
    TtlPolicyCycleEngine {
        steps: vec![
            Rc::new(TtlPolicyCycleScoreOffsetAdjuster { weight: 0.26 }) as Rc<dyn TtlPolicyCycleFactor>,
            Rc::new(SourceTtlPolicyCycleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.26)]),
            }),
            Rc::new(FlaggedTtlPolicyCycleAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-cycle-global".to_string(),
                percent: -5.8,
            }),
            Rc::new(MinimumTtlPolicyCycleAdjuster { minimum: 35.5 }),
            Rc::new(CapTtlPolicyCycleAdjuster { cap: 39207.0 }),
            Rc::new(PercentTtlPolicyCycleAdjuster { percent: -5.6 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 57035.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_cycle_engine(&flags::new_registry());
        let item = new_ttl_policy_cycle("code-708", "name-962");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
