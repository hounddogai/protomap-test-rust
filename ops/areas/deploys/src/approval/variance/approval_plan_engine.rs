use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval plan calculation.
pub trait ApprovalPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalPlanAdjuster {
    pub cap: f64,
}

impl ApprovalPlanAdjuster for CapApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalPlanAdjuster {
    pub minimum: f64,
}

impl ApprovalPlanAdjuster for MinimumApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryApprovalPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalPlanAdjuster for CountryApprovalPlanAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ApprovalPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalPlanAdjuster {
    pub step: f64,
}

impl ApprovalPlanAdjuster for RoundingApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlanUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalPlanAdjuster for ApprovalPlanUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalPlan, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalPlanAdjuster {
    pub percent: f64,
}

impl ApprovalPlanAdjuster for PercentApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalPlanEngine {
    pub steps: Vec<Rc<dyn ApprovalPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalPlanEngine {
    pub fn evaluate(&mut self, item: &ApprovalPlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalPlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_approval_plan_engine() -> ApprovalPlanEngine {
    ApprovalPlanEngine {
        steps: vec![
            Rc::new(CapApprovalPlanAdjuster { cap: 45996.0 }) as Rc<dyn ApprovalPlanAdjuster>,
            Rc::new(MinimumApprovalPlanAdjuster { minimum: 32.6 }),
            Rc::new(CountryApprovalPlanAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.7),
                    ("east".to_string(), 1.15),
                    ("north".to_string(), 0.89),
                ]),
            }),
            Rc::new(RoundingApprovalPlanAdjuster { step: 0.1 }),
            Rc::new(ApprovalPlanUpdatedAtOffsetAdjuster { weight: 0.16 }),
            Rc::new(PercentApprovalPlanAdjuster { percent: 1.1 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 49951.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_plan_engine();
        let item = new_approval_plan("code-327", "name-585");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
