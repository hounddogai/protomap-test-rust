use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global egress notice calculation.
pub trait GlobalEgressNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalEgressNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalEgressNoticeAdjuster {
    pub minimum: f64,
}

impl GlobalEgressNoticeFactor for MinimumGlobalEgressNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalEgressNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalEgressNoticeAdjuster {
    pub step: f64,
}

impl GlobalEgressNoticeFactor for RoundingGlobalEgressNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalEgressNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalEgressNoticeAdjuster {
    pub percent: f64,
}

impl GlobalEgressNoticeFactor for PercentGlobalEgressNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalEgressNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressNoticeMarginOffsetAdjuster {
    pub weight: f64,
}

impl GlobalEgressNoticeFactor for GlobalEgressNoticeMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalEgressNotice, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the global egress notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalEgressNoticeEngine {
    pub steps: Vec<Rc<dyn GlobalEgressNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalEgressNoticeEngine {
    pub fn evaluate(&mut self, item: &GlobalEgressNotice) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[GlobalEgressNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_egress_notice_engine() -> GlobalEgressNoticeEngine {
    GlobalEgressNoticeEngine {
        steps: vec![
            Rc::new(MinimumGlobalEgressNoticeAdjuster { minimum: 38.0 }) as Rc<dyn GlobalEgressNoticeFactor>,
            Rc::new(RoundingGlobalEgressNoticeAdjuster { step: 5.0 }),
            Rc::new(PercentGlobalEgressNoticeAdjuster { percent: 16.8 }),
            Rc::new(GlobalEgressNoticeMarginOffsetAdjuster { weight: 1.58 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 59423.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_egress_notice_engine();
        let item = new_global_egress_notice("code-269", "name-448");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
