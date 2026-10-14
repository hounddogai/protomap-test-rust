use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling identity plan calculation.
pub trait RollingIdentityPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingIdentityPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingIdentityPlanAdjuster {
    pub percent: f64,
}

impl RollingIdentityPlanModifier for PercentRollingIdentityPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingIdentityPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRollingIdentityPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingIdentityPlanModifier for TitleRollingIdentityPlanAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RollingIdentityPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlanRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RollingIdentityPlanModifier for RollingIdentityPlanRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RollingIdentityPlan, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingIdentityPlanAdjuster {
    pub cap: f64,
}

impl RollingIdentityPlanModifier for CapRollingIdentityPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingIdentityPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingIdentityPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingIdentityPlanModifier for TieredRollingIdentityPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingIdentityPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling identity plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingIdentityPlanEngine {
    pub steps: Vec<Rc<dyn RollingIdentityPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingIdentityPlanEngine {
    pub fn compute(&mut self, item: &RollingIdentityPlan) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingIdentityPlanModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingIdentityPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_identity_plan_engine() -> RollingIdentityPlanEngine {
    RollingIdentityPlanEngine {
        steps: vec![
            Rc::new(PercentRollingIdentityPlanAdjuster { percent: -9.5 }) as Rc<dyn RollingIdentityPlanModifier>,
            Rc::new(TitleRollingIdentityPlanAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.03), ("north".to_string(), 0.71)]),
            }),
            Rc::new(RollingIdentityPlanRevisionOffsetAdjuster { weight: 1.48 }),
            Rc::new(CapRollingIdentityPlanAdjuster { cap: 11721.0 }),
            Rc::new(TieredRollingIdentityPlanAdjuster {
                bounds: vec![1312.0, 9631.0],
                rates: vec![1.17, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 41044.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_identity_plan_engine();
        let item = new_rolling_identity_plan("code-549", "title-834");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
