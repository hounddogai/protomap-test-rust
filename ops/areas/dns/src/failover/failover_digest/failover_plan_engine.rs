use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover plan calculation.
pub trait FailoverPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFailoverPlanAdjuster {
    pub percent: f64,
}

impl FailoverPlanFactor for PercentFailoverPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FailoverPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFailoverPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverPlanFactor for NameFailoverPlanAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FailoverPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPlanDiscountOffsetAdjuster {
    pub weight: f64,
}

impl FailoverPlanFactor for FailoverPlanDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverPlan, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the failover plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverPlanEngine {
    pub steps: Vec<Rc<dyn FailoverPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverPlanEngine {
    pub fn compute(&mut self, item: &FailoverPlan) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[FailoverPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_failover_plan_engine() -> FailoverPlanEngine {
    FailoverPlanEngine {
        steps: vec![
            Rc::new(PercentFailoverPlanAdjuster { percent: 20.7 }) as Rc<dyn FailoverPlanFactor>,
            Rc::new(NameFailoverPlanAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.72),
                    ("south".to_string(), 0.81),
                    ("north".to_string(), 1.24),
                ]),
            }),
            Rc::new(FailoverPlanDiscountOffsetAdjuster { weight: 0.7 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 70742.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_plan_engine();
        let item = new_failover_plan("code-204", "name-629");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
