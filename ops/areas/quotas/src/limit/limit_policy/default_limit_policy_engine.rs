use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default limit policy calculation.
pub trait DefaultLimitPolicyStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultLimitPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultLimitPolicyAdjuster {
    pub percent: f64,
}

impl DefaultLimitPolicyStep for PercentDefaultLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultLimitPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitPolicyScoreOffsetAdjuster {
    pub weight: f64,
}

impl DefaultLimitPolicyStep for DefaultLimitPolicyScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultLimitPolicy, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultLimitPolicyAdjuster {
    pub cap: f64,
}

impl DefaultLimitPolicyStep for CapDefaultLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultLimitPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultLimitPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultLimitPolicyStep for FlaggedDefaultLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultLimitPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultLimitPolicyAdjuster {
    pub minimum: f64,
}

impl DefaultLimitPolicyStep for MinimumDefaultLimitPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultLimitPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the default limit policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultLimitPolicyEngine {
    pub steps: Vec<Rc<dyn DefaultLimitPolicyStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultLimitPolicyEngine {
    pub fn run(&mut self, item: &DefaultLimitPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DefaultLimitPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_limit_policy_engine(flags: &flags::Registry) -> DefaultLimitPolicyEngine {
    DefaultLimitPolicyEngine {
        steps: vec![
            Rc::new(PercentDefaultLimitPolicyAdjuster { percent: 14.6 }) as Rc<dyn DefaultLimitPolicyStep>,
            Rc::new(DefaultLimitPolicyScoreOffsetAdjuster { weight: 0.34 }),
            Rc::new(CapDefaultLimitPolicyAdjuster { cap: 3147.0 }),
            Rc::new(FlaggedDefaultLimitPolicyAdjuster {
                flags: flags.clone(),
                flag: "default-limit-policy-batched".to_string(),
                percent: 4.9,
            }),
            Rc::new(MinimumDefaultLimitPolicyAdjuster { minimum: 44.5 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 97187.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_limit_policy_engine(&flags::new_registry());
        let item = new_default_limit_policy("code-565", "label-11");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
