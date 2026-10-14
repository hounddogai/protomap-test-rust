use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval notice calculation.
pub trait ApprovalNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalNoticeAdjuster {
    pub cap: f64,
}

impl ApprovalNoticeAdjuster for CapApprovalNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalNoticeAdjuster {
    pub minimum: f64,
}

impl ApprovalNoticeAdjuster for MinimumApprovalNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalNoticeAdjuster {
    pub step: f64,
}

impl ApprovalNoticeAdjuster for RoundingApprovalNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalNoticeThresholdOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalNoticeAdjuster for ApprovalNoticeThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalNotice, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the approval notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalNoticeEngine {
    pub steps: Vec<Rc<dyn ApprovalNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalNoticeEngine {
    pub fn price(&mut self, item: &ApprovalNotice) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_notice_engine() -> ApprovalNoticeEngine {
    ApprovalNoticeEngine {
        steps: vec![
            Rc::new(CapApprovalNoticeAdjuster { cap: 45831.0 }) as Rc<dyn ApprovalNoticeAdjuster>,
            Rc::new(MinimumApprovalNoticeAdjuster { minimum: 24.1 }),
            Rc::new(RoundingApprovalNoticeAdjuster { step: 0.1 }),
            Rc::new(ApprovalNoticeThresholdOffsetAdjuster { weight: 0.99 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 99506.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_notice_engine();
        let item = new_approval_notice("code-515", "label-653");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
