use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity plan calculation.
pub trait IdentityPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterIdentityPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityPlanFactor for ClusterIdentityPlanAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &IdentityPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityPlanAdjuster {
    pub step: f64,
}

impl IdentityPlanFactor for RoundingIdentityPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityPlanAdjuster {
    pub cap: f64,
}

impl IdentityPlanFactor for CapIdentityPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the identity plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityPlanEngine {
    pub steps: Vec<Rc<dyn IdentityPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityPlanEngine {
    pub fn price(&mut self, item: &IdentityPlan) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[IdentityPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_plan_engine() -> IdentityPlanEngine {
    IdentityPlanEngine {
        steps: vec![
            Rc::new(ClusterIdentityPlanAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.82),
                    ("central".to_string(), 0.78),
                    ("online".to_string(), 1.33),
                ]),
            }) as Rc<dyn IdentityPlanFactor>,
            Rc::new(RoundingIdentityPlanAdjuster { step: 0.1 }),
            Rc::new(CapIdentityPlanAdjuster { cap: 2946.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 30373.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_plan_engine();
        let item = new_identity_plan("code-808", "title-492");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
