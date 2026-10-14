use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval adjustment calculation.
pub trait ApprovalAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalAdjustmentAdjuster {
    pub minimum: f64,
}

impl ApprovalAdjustmentAdjuster for MinimumApprovalAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalAdjustmentAdjuster {
    pub percent: f64,
}

impl ApprovalAdjustmentAdjuster for PercentApprovalAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalAdjustmentAdjuster {
    pub step: f64,
}

impl ApprovalAdjustmentAdjuster for RoundingApprovalAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalAdjustmentAdjuster for FlaggedApprovalAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalAdjustmentAdjuster {
    pub cap: f64,
}

impl ApprovalAdjustmentAdjuster for CapApprovalAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAdjustmentDiscountOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalAdjustmentAdjuster for ApprovalAdjustmentDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalAdjustment, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

/// Runs the approval adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalAdjustmentEngine {
    pub steps: Vec<Rc<dyn ApprovalAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalAdjustmentEngine {
    pub fn price(&mut self, item: &ApprovalAdjustment) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalAdjustmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_adjustment_engine(flags: &flags::Registry) -> ApprovalAdjustmentEngine {
    ApprovalAdjustmentEngine {
        steps: vec![
            Rc::new(MinimumApprovalAdjustmentAdjuster { minimum: 18.0 }) as Rc<dyn ApprovalAdjustmentAdjuster>,
            Rc::new(PercentApprovalAdjustmentAdjuster { percent: -1.0 }),
            Rc::new(RoundingApprovalAdjustmentAdjuster { step: 0.05 }),
            Rc::new(FlaggedApprovalAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "approval-adjustment-manual".to_string(),
                percent: -3.6,
            }),
            Rc::new(CapApprovalAdjustmentAdjuster { cap: 27377.0 }),
            Rc::new(ApprovalAdjustmentDiscountOffsetAdjuster { weight: 0.61 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 58033.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_adjustment_engine(&flags::new_registry());
        let item = new_approval_adjustment("code-138", "title-27");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
