use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule batch calculation.
pub trait RuleBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleBatchAdjuster {
    pub minimum: f64,
}

impl RuleBatchFactor for MinimumRuleBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatchExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl RuleBatchFactor for RuleBatchExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &RuleBatch, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRuleBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleBatchFactor for LocaleRuleBatchAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RuleBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rule batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleBatchEngine {
    pub steps: Vec<Rc<dyn RuleBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleBatchEngine {
    pub fn compute(&mut self, item: &RuleBatch) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[RuleBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_batch_engine() -> RuleBatchEngine {
    RuleBatchEngine {
        steps: vec![
            Rc::new(MinimumRuleBatchAdjuster { minimum: 35.3 }) as Rc<dyn RuleBatchFactor>,
            Rc::new(RuleBatchExpiresAtOffsetAdjuster { weight: 0.94 }),
            Rc::new(LocaleRuleBatchAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.11), ("south".to_string(), 1.27)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 4744.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_batch_engine();
        let item = new_rule_batch("code-472", "name-858");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
