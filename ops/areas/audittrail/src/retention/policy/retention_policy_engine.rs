use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention policy calculation.
pub trait RetentionPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionPolicyAdjuster for FlaggedRetentionPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionPolicyAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RetentionPolicyAdjuster for RetentionPolicyAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionPolicy, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionPolicyAdjuster {
    pub step: f64,
}

impl RetentionPolicyAdjuster for RoundingRetentionPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the retention policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionPolicyEngine {
    pub steps: Vec<Rc<dyn RetentionPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionPolicyEngine {
    pub fn evaluate(&mut self, item: &RetentionPolicy) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[RetentionPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_policy_engine(flags: &flags::Registry) -> RetentionPolicyEngine {
    RetentionPolicyEngine {
        steps: vec![
            Rc::new(FlaggedRetentionPolicyAdjuster {
                flags: flags.clone(),
                flag: "retention-policy-regional".to_string(),
                percent: 8.9,
            }) as Rc<dyn RetentionPolicyAdjuster>,
            Rc::new(RetentionPolicyAttemptsOffsetAdjuster { weight: 0.64 }),
            Rc::new(RoundingRetentionPolicyAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 58458.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_policy_engine(&flags::new_registry());
        let item = new_retention_policy("code-458", "label-716");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
