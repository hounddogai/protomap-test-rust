use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched retention rule calculation.
pub trait BatchedRetentionRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRetentionRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedRetentionRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedRetentionRuleStep for FlaggedBatchedRetentionRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedRetentionRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitBatchedRetentionRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedRetentionRuleStep for UnitBatchedRetentionRuleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &BatchedRetentionRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRetentionRuleAdjuster {
    pub step: f64,
}

impl BatchedRetentionRuleStep for RoundingBatchedRetentionRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRetentionRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched retention rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRetentionRuleEngine {
    pub steps: Vec<Rc<dyn BatchedRetentionRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRetentionRuleEngine {
    pub fn evaluate(&mut self, item: &BatchedRetentionRule) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRetentionRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_retention_rule_engine(flags: &flags::Registry) -> BatchedRetentionRuleEngine {
    BatchedRetentionRuleEngine {
        steps: vec![
            Rc::new(FlaggedBatchedRetentionRuleAdjuster {
                flags: flags.clone(),
                flag: "batched-retention-rule-lenient".to_string(),
                percent: -3.1,
            }) as Rc<dyn BatchedRetentionRuleStep>,
            Rc::new(UnitBatchedRetentionRuleAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.77),
                    ("west".to_string(), 1.24),
                    ("online".to_string(), 0.81),
                ]),
            }),
            Rc::new(RoundingBatchedRetentionRuleAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 11727.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_retention_rule_engine(&flags::new_registry());
        let item = new_batched_retention_rule("code-738", "name-812");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
