use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollback plan calculation.
pub trait RollbackPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPlanBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl RollbackPlanFactor for RollbackPlanBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackPlan, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackPlanAdjuster {
    pub cap: f64,
}

impl RollbackPlanFactor for CapRollbackPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackPlanAdjuster {
    pub percent: f64,
}

impl RollbackPlanFactor for PercentRollbackPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollbackPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackPlanFactor for LabelRollbackPlanAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollbackPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackPlanFactor for FlaggedRollbackPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackPlanEngine {
    pub steps: Vec<Rc<dyn RollbackPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackPlanEngine {
    pub fn compute(&mut self, item: &RollbackPlan) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RollbackPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_plan_engine(flags: &flags::Registry) -> RollbackPlanEngine {
    RollbackPlanEngine {
        steps: vec![
            Rc::new(RollbackPlanBatchSizeOffsetAdjuster { weight: 1.44 }) as Rc<dyn RollbackPlanFactor>,
            Rc::new(CapRollbackPlanAdjuster { cap: 4712.0 }),
            Rc::new(PercentRollbackPlanAdjuster { percent: 14.6 }),
            Rc::new(LabelRollbackPlanAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.81), ("east".to_string(), 1.01)]),
            }),
            Rc::new(FlaggedRollbackPlanAdjuster {
                flags: flags.clone(),
                flag: "rollback-plan-fallback".to_string(),
                percent: 8.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 99433.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_plan_engine(&flags::new_registry());
        let item = new_rollback_plan("code-858", "label-259");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
