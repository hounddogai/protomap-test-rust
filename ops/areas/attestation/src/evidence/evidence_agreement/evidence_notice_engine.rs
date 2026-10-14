use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence notice calculation.
pub trait EvidenceNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceNotice, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceNoticeFactor for FlaggedEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceNoticeAdjuster {
    pub percent: f64,
}

impl EvidenceNoticeFactor for PercentEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceNoticeAdjuster {
    pub cap: f64,
}

impl EvidenceNoticeFactor for CapEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the evidence notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceNoticeEngine {
    pub steps: Vec<Rc<dyn EvidenceNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceNoticeEngine {
    pub fn compute(&mut self, item: &EvidenceNotice) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_evidence_notice_engine(flags: &flags::Registry) -> EvidenceNoticeEngine {
    EvidenceNoticeEngine {
        steps: vec![
            Rc::new(FlaggedEvidenceNoticeAdjuster {
                flags: flags.clone(),
                flag: "evidence-notice-batched".to_string(),
                percent: -6.6,
            }) as Rc<dyn EvidenceNoticeFactor>,
            Rc::new(PercentEvidenceNoticeAdjuster { percent: -7.2 }),
            Rc::new(CapEvidenceNoticeAdjuster { cap: 5876.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 83674.8,
    }
}
