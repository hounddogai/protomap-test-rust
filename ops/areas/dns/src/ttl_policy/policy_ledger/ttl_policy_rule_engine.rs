use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the ttl policy rule calculation.
pub trait TtlPolicyRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyRuleAdjuster {
    pub cap: f64,
}

impl TtlPolicyRuleModifier for CapTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyRuleAdjuster {
    pub minimum: f64,
}

impl TtlPolicyRuleModifier for MinimumTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelTtlPolicyRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyRuleModifier for LabelTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &TtlPolicyRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyRuleModifier for TieredTtlPolicyRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRuleSequenceOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyRuleModifier for TtlPolicyRuleSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyRule, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the ttl policy rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyRuleEngine {
    pub steps: Vec<Rc<dyn TtlPolicyRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyRuleEngine {
    pub fn run(&mut self, item: &TtlPolicyRule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyRuleModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[TtlPolicyRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_rule_engine() -> TtlPolicyRuleEngine {
    TtlPolicyRuleEngine {
        steps: vec![
            Rc::new(CapTtlPolicyRuleAdjuster { cap: 13607.0 }) as Rc<dyn TtlPolicyRuleModifier>,
            Rc::new(MinimumTtlPolicyRuleAdjuster { minimum: 10.3 }),
            Rc::new(LabelTtlPolicyRuleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.79),
                    ("central".to_string(), 0.83),
                    ("retail".to_string(), 0.9),
                ]),
            }),
            Rc::new(TieredTtlPolicyRuleAdjuster {
                bounds: vec![4695.0, 6550.0],
                rates: vec![1.0, 1.07],
            }),
            Rc::new(TtlPolicyRuleSequenceOffsetAdjuster { weight: 0.58 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 27763.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_rule_engine();
        let item = new_ttl_policy_rule("code-571", "label-736");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
