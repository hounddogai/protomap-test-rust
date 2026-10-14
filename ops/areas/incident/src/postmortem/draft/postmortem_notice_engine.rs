use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the postmortem notice calculation.
pub trait PostmortemNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemNotice, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemNoticeModifier for FlaggedPostmortemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemNoticeRatioOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemNoticeModifier for PostmortemNoticeRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemNotice, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPostmortemNoticeAdjuster {
    pub step: f64,
}

impl PostmortemNoticeModifier for RoundingPostmortemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PostmortemNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemNoticeAdjuster {
    pub minimum: f64,
}

impl PostmortemNoticeModifier for MinimumPostmortemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPostmortemNoticeAdjuster {
    pub cap: f64,
}

impl PostmortemNoticeModifier for CapPostmortemNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PostmortemNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the postmortem notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemNoticeEngine {
    pub steps: Vec<Rc<dyn PostmortemNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemNoticeEngine {
    pub fn run(&mut self, item: &PostmortemNotice) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_postmortem_notice_engine(flags: &flags::Registry) -> PostmortemNoticeEngine {
    PostmortemNoticeEngine {
        steps: vec![
            Rc::new(FlaggedPostmortemNoticeAdjuster {
                flags: flags.clone(),
                flag: "postmortem-notice-global".to_string(),
                percent: 8.2,
            }) as Rc<dyn PostmortemNoticeModifier>,
            Rc::new(PostmortemNoticeRatioOffsetAdjuster { weight: 1.7 }),
            Rc::new(RoundingPostmortemNoticeAdjuster { step: 5.0 }),
            Rc::new(MinimumPostmortemNoticeAdjuster { minimum: 11.9 }),
            Rc::new(CapPostmortemNoticeAdjuster { cap: 21534.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 39796.1,
    }
}
