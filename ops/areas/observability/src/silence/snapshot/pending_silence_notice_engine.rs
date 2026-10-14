use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending silence notice calculation.
pub trait PendingSilenceNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSilenceNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingSilenceNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingSilenceNoticeAdjuster for TieredPendingSilenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingSilenceNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingSilenceNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingSilenceNoticeAdjuster for SourcePendingSilenceNoticeAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingSilenceNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingSilenceNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingSilenceNoticeAdjuster for FlaggedPendingSilenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingSilenceNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending silence notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSilenceNoticeEngine {
    pub steps: Vec<Rc<dyn PendingSilenceNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSilenceNoticeEngine {
    pub fn compute(&mut self, item: &PendingSilenceNotice) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingSilenceNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_silence_notice_engine(flags: &flags::Registry) -> PendingSilenceNoticeEngine {
    PendingSilenceNoticeEngine {
        steps: vec![
            Rc::new(TieredPendingSilenceNoticeAdjuster {
                bounds: vec![324.0, 887.0, 1734.0, 8916.0],
                rates: vec![1.18, 0.85, 1.16, 1.08],
            }) as Rc<dyn PendingSilenceNoticeAdjuster>,
            Rc::new(SourcePendingSilenceNoticeAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.07)]),
            }),
            Rc::new(FlaggedPendingSilenceNoticeAdjuster {
                flags: flags.clone(),
                flag: "pending-silence-notice-scheduled".to_string(),
                percent: -0.1,
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 68851.9,
    }
}
