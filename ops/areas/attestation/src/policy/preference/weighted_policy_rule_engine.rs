use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted policy rule calculation.
pub trait WeightedPolicyRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedPolicyRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyRuleCostOffsetAdjuster {
    pub weight: f64,
}

impl WeightedPolicyRuleFactor for WeightedPolicyRuleCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedPolicyRule, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentWeightedPolicyRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedPolicyRuleFactor for SegmentWeightedPolicyRuleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &WeightedPolicyRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedPolicyRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedPolicyRuleFactor for TieredWeightedPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedPolicyRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted policy rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedPolicyRuleEngine {
    pub steps: Vec<Rc<dyn WeightedPolicyRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedPolicyRuleEngine {
    pub fn price(&mut self, item: &WeightedPolicyRule) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[WeightedPolicyRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_policy_rule_engine() -> WeightedPolicyRuleEngine {
    WeightedPolicyRuleEngine {
        steps: vec![
            Rc::new(WeightedPolicyRuleCostOffsetAdjuster { weight: 1.9 }) as Rc<dyn WeightedPolicyRuleFactor>,
            Rc::new(SegmentWeightedPolicyRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.32), ("east".to_string(), 1.19)]),
            }),
            Rc::new(TieredWeightedPolicyRuleAdjuster {
                bounds: vec![2081.0, 4738.0, 5232.0, 9062.0],
                rates: vec![0.81, 1.17, 0.86, 0.82],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 43135.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_policy_rule_engine();
        let item = new_weighted_policy_rule("code-820", "name-153");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
