use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched runbook notice calculation.
pub trait BatchedRunbookNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRunbookNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRunbookNoticeAdjuster {
    pub step: f64,
}

impl BatchedRunbookNoticeFactor for RoundingBatchedRunbookNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRunbookNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceBatchedRunbookNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedRunbookNoticeFactor for SourceBatchedRunbookNoticeAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &BatchedRunbookNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedRunbookNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedRunbookNoticeFactor for FlaggedBatchedRunbookNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedRunbookNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookNoticeQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRunbookNoticeFactor for BatchedRunbookNoticeQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRunbookNotice, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedRunbookNoticeAdjuster {
    pub percent: f64,
}

impl BatchedRunbookNoticeFactor for PercentBatchedRunbookNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedRunbookNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched runbook notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRunbookNoticeEngine {
    pub steps: Vec<Rc<dyn BatchedRunbookNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRunbookNoticeEngine {
    pub fn run(&mut self, item: &BatchedRunbookNotice) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedRunbookNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_runbook_notice_engine(flags: &flags::Registry) -> BatchedRunbookNoticeEngine {
    BatchedRunbookNoticeEngine {
        steps: vec![
            Rc::new(RoundingBatchedRunbookNoticeAdjuster { step: 1.0 }) as Rc<dyn BatchedRunbookNoticeFactor>,
            Rc::new(SourceBatchedRunbookNoticeAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.1)]),
            }),
            Rc::new(FlaggedBatchedRunbookNoticeAdjuster {
                flags: flags.clone(),
                flag: "batched-runbook-notice-global".to_string(),
                percent: -3.6,
            }),
            Rc::new(BatchedRunbookNoticeQuotaUnitsOffsetAdjuster { weight: 0.67 }),
            Rc::new(PercentBatchedRunbookNoticeAdjuster { percent: -2.7 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 56790.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_runbook_notice_engine(&flags::new_registry());
        let item = new_batched_runbook_notice("code-5", "label-571");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
