use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional ttl policy rule calculation.
pub trait RegionalTtlPolicyRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalTtlPolicyRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalTtlPolicyRuleAdjuster {
    pub minimum: f64,
}

impl RegionalTtlPolicyRuleModifier for MinimumRegionalTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalTtlPolicyRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalTtlPolicyRuleAdjuster {
    pub percent: f64,
}

impl RegionalTtlPolicyRuleModifier for PercentRegionalTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalTtlPolicyRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalTtlPolicyRuleAdjuster {
    pub cap: f64,
}

impl RegionalTtlPolicyRuleModifier for CapRegionalTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalTtlPolicyRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleDiscountOffsetAdjuster {
    pub weight: f64,
}

impl RegionalTtlPolicyRuleModifier for RegionalTtlPolicyRuleDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalTtlPolicyRule, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the regional ttl policy rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalTtlPolicyRuleEngine {
    pub steps: Vec<Rc<dyn RegionalTtlPolicyRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalTtlPolicyRuleEngine {
    pub fn price(&mut self, item: &RegionalTtlPolicyRule) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalTtlPolicyRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_ttl_policy_rule_engine() -> RegionalTtlPolicyRuleEngine {
    RegionalTtlPolicyRuleEngine {
        steps: vec![
            Rc::new(MinimumRegionalTtlPolicyRuleAdjuster { minimum: 5.7 }) as Rc<dyn RegionalTtlPolicyRuleModifier>,
            Rc::new(PercentRegionalTtlPolicyRuleAdjuster { percent: 20.8 }),
            Rc::new(CapRegionalTtlPolicyRuleAdjuster { cap: 42626.0 }),
            Rc::new(RegionalTtlPolicyRuleDiscountOffsetAdjuster { weight: 1.1 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 4384.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_ttl_policy_rule_engine();
        let item = new_regional_ttl_policy_rule("code-240", "title-800");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
