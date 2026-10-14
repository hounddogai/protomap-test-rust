use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan rule calculation.
pub trait PlanRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanRuleAdjuster {
    pub minimum: f64,
}

impl PlanRuleModifier for MinimumPlanRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePlanRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanRuleModifier for TitlePlanRuleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PlanRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRuleBurstOffsetAdjuster {
    pub weight: f64,
}

impl PlanRuleModifier for PlanRuleBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &PlanRule, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanRuleAdjuster {
    pub step: f64,
}

impl PlanRuleModifier for RoundingPlanRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanRuleModifier for TieredPlanRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the plan rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanRuleEngine {
    pub steps: Vec<Rc<dyn PlanRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanRuleEngine {
    pub fn price(&mut self, item: &PlanRule) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PlanRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_rule_engine() -> PlanRuleEngine {
    PlanRuleEngine {
        steps: vec![
            Rc::new(MinimumPlanRuleAdjuster { minimum: 2.6 }) as Rc<dyn PlanRuleModifier>,
            Rc::new(TitlePlanRuleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.01), ("east".to_string(), 0.8)]),
            }),
            Rc::new(PlanRuleBurstOffsetAdjuster { weight: 0.58 }),
            Rc::new(RoundingPlanRuleAdjuster { step: 0.5 }),
            Rc::new(TieredPlanRuleAdjuster {
                bounds: vec![3534.0, 4384.0, 9273.0],
                rates: vec![0.84, 1.18, 1.02],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 90637.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_rule_engine();
        let item = new_plan_rule("code-555", "title-434");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
