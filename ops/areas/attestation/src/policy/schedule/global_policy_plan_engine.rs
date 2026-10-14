use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global policy plan calculation.
pub trait GlobalPolicyPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPolicyPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalPolicyPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalPolicyPlanAdjuster for TieredGlobalPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalPolicyPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPolicyPlanAdjuster {
    pub minimum: f64,
}

impl GlobalPolicyPlanAdjuster for MinimumGlobalPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPolicyPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPolicyPlanAdjuster {
    pub cap: f64,
}

impl GlobalPolicyPlanAdjuster for CapGlobalPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPolicyPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalPolicyPlanAdjuster {
    pub percent: f64,
}

impl GlobalPolicyPlanAdjuster for PercentGlobalPolicyPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalPolicyPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterGlobalPolicyPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPolicyPlanAdjuster for ClusterGlobalPolicyPlanAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &GlobalPolicyPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlanScoreOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPolicyPlanAdjuster for GlobalPolicyPlanScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPolicyPlan, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the global policy plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPolicyPlanEngine {
    pub steps: Vec<Rc<dyn GlobalPolicyPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPolicyPlanEngine {
    pub fn evaluate(&mut self, item: &GlobalPolicyPlan) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPolicyPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_policy_plan_engine() -> GlobalPolicyPlanEngine {
    GlobalPolicyPlanEngine {
        steps: vec![
            Rc::new(TieredGlobalPolicyPlanAdjuster {
                bounds: vec![111.0, 4647.0, 8620.0, 9150.0],
                rates: vec![1.16, 1.17, 0.92, 0.9],
            }) as Rc<dyn GlobalPolicyPlanAdjuster>,
            Rc::new(MinimumGlobalPolicyPlanAdjuster { minimum: 45.6 }),
            Rc::new(CapGlobalPolicyPlanAdjuster { cap: 15433.0 }),
            Rc::new(PercentGlobalPolicyPlanAdjuster { percent: -7.1 }),
            Rc::new(ClusterGlobalPolicyPlanAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.99), ("west".to_string(), 0.75)]),
            }),
            Rc::new(GlobalPolicyPlanScoreOffsetAdjuster { weight: 0.76 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 19026.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_policy_plan_engine();
        let item = new_global_policy_plan("code-878", "name-442");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
