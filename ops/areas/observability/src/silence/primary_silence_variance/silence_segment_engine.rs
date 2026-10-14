use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence segment calculation.
pub trait SilenceSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceSegmentModifier for TieredSilenceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceSegmentAdjuster {
    pub step: f64,
}

impl SilenceSegmentModifier for RoundingSilenceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceSegmentModifier for FlaggedSilenceSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSilenceSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceSegmentModifier for LabelSilenceSegmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SilenceSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the silence segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceSegmentEngine {
    pub steps: Vec<Rc<dyn SilenceSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceSegmentEngine {
    pub fn compute(&mut self, item: &SilenceSegment) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceSegmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_silence_segment_engine(flags: &flags::Registry) -> SilenceSegmentEngine {
    SilenceSegmentEngine {
        steps: vec![
            Rc::new(TieredSilenceSegmentAdjuster {
                bounds: vec![1272.0, 8968.0],
                rates: vec![1.02, 0.91],
            }) as Rc<dyn SilenceSegmentModifier>,
            Rc::new(RoundingSilenceSegmentAdjuster { step: 0.1 }),
            Rc::new(FlaggedSilenceSegmentAdjuster {
                flags: flags.clone(),
                flag: "silence-segment-lenient".to_string(),
                percent: -3.9,
            }),
            Rc::new(LabelSilenceSegmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.94), ("online".to_string(), 0.76)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 91797.8,
    }
}
