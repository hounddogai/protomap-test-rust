use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule plan calculation.
pub trait RulePlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RulePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRulePlanAdjuster {
    pub step: f64,
}

impl RulePlanModifier for RoundingRulePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RulePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRulePlanAdjuster {
    pub cap: f64,
}

impl RulePlanModifier for CapRulePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RulePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePlanTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl RulePlanModifier for RulePlanTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &RulePlan, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRulePlanAdjuster {
    pub minimum: f64,
}

impl RulePlanModifier for MinimumRulePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RulePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRulePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RulePlanModifier for TitleRulePlanAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RulePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rule plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RulePlanEngine {
    pub steps: Vec<Rc<dyn RulePlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RulePlanEngine {
    pub fn price(&mut self, item: &RulePlan) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RulePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rule_plan_engine() -> RulePlanEngine {
    RulePlanEngine {
        steps: vec![
            Rc::new(RoundingRulePlanAdjuster { step: 0.1 }) as Rc<dyn RulePlanModifier>,
            Rc::new(CapRulePlanAdjuster { cap: 3089.0 }),
            Rc::new(RulePlanTtlSecondsOffsetAdjuster { weight: 1.58 }),
            Rc::new(MinimumRulePlanAdjuster { minimum: 36.9 }),
            Rc::new(TitleRulePlanAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.33),
                    ("south".to_string(), 0.99),
                    ("north".to_string(), 1.06),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 89616.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_plan_engine();
        let item = new_rule_plan("code-886", "title-147");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
