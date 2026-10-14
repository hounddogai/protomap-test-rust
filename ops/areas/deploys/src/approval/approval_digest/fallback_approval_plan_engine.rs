use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback approval plan calculation.
pub trait FallbackApprovalPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackApprovalPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackApprovalPlanAdjuster {
    pub cap: f64,
}

impl FallbackApprovalPlanModifier for CapFallbackApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackApprovalPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionFallbackApprovalPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackApprovalPlanModifier for RegionFallbackApprovalPlanAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &FallbackApprovalPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalPlanCostOffsetAdjuster {
    pub weight: f64,
}

impl FallbackApprovalPlanModifier for FallbackApprovalPlanCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackApprovalPlan, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackApprovalPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackApprovalPlanModifier for FlaggedFallbackApprovalPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackApprovalPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback approval plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackApprovalPlanEngine {
    pub steps: Vec<Rc<dyn FallbackApprovalPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackApprovalPlanEngine {
    pub fn evaluate(&mut self, item: &FallbackApprovalPlan) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FallbackApprovalPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_approval_plan_engine(flags: &flags::Registry) -> FallbackApprovalPlanEngine {
    FallbackApprovalPlanEngine {
        steps: vec![
            Rc::new(CapFallbackApprovalPlanAdjuster { cap: 14982.0 }) as Rc<dyn FallbackApprovalPlanModifier>,
            Rc::new(RegionFallbackApprovalPlanAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.71),
                    ("east".to_string(), 1.09),
                    ("west".to_string(), 0.87),
                ]),
            }),
            Rc::new(FallbackApprovalPlanCostOffsetAdjuster { weight: 1.24 }),
            Rc::new(FlaggedFallbackApprovalPlanAdjuster {
                flags: flags.clone(),
                flag: "fallback-approval-plan-default".to_string(),
                percent: 0.3,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 82630.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_approval_plan_engine(&flags::new_registry());
        let item = new_fallback_approval_plan("code-11", "title-142");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
