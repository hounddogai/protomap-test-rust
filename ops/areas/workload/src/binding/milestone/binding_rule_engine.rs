use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding rule calculation.
pub trait BindingRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBindingRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingRuleFactor for SegmentBindingRuleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &BindingRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingRuleAdjuster {
    pub minimum: f64,
}

impl BindingRuleFactor for MinimumBindingRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingRuleAdjuster {
    pub cap: f64,
}

impl BindingRuleFactor for CapBindingRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingRuleFactor for TieredBindingRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRuleStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl BindingRuleFactor for BindingRuleStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &BindingRule, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the binding rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingRuleEngine {
    pub steps: Vec<Rc<dyn BindingRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingRuleEngine {
    pub fn evaluate(&mut self, item: &BindingRule) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[BindingRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_rule_engine() -> BindingRuleEngine {
    BindingRuleEngine {
        steps: vec![
            Rc::new(SegmentBindingRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.79), ("central".to_string(), 1.32)]),
            }) as Rc<dyn BindingRuleFactor>,
            Rc::new(MinimumBindingRuleAdjuster { minimum: 5.8 }),
            Rc::new(CapBindingRuleAdjuster { cap: 46018.0 }),
            Rc::new(TieredBindingRuleAdjuster {
                bounds: vec![3083.0, 3583.0, 6311.0, 6969.0],
                rates: vec![1.18, 0.98, 0.97, 0.94],
            }),
            Rc::new(BindingRuleStartsAtOffsetAdjuster { weight: 1.66 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 88235.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_rule_engine();
        let item = new_binding_rule("code-649", "name-711");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
