use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe notice calculation.
pub trait ProbeNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingProbeNoticeAdjuster {
    pub step: f64,
}

impl ProbeNoticeAdjuster for RoundingProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ProbeNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeNoticeAdjuster for FlaggedProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeNoticeAdjuster {
    pub percent: f64,
}

impl ProbeNoticeAdjuster for PercentProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeNoticeAdjuster {
    pub minimum: f64,
}

impl ProbeNoticeAdjuster for MinimumProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelProbeNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeNoticeAdjuster for LabelProbeNoticeAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ProbeNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeNoticeAdjuster {
    pub cap: f64,
}

impl ProbeNoticeAdjuster for CapProbeNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the probe notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeNoticeEngine {
    pub steps: Vec<Rc<dyn ProbeNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeNoticeEngine {
    pub fn run(&mut self, item: &ProbeNotice) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[ProbeNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_probe_notice_engine(flags: &flags::Registry) -> ProbeNoticeEngine {
    ProbeNoticeEngine {
        steps: vec![
            Rc::new(RoundingProbeNoticeAdjuster { step: 0.1 }) as Rc<dyn ProbeNoticeAdjuster>,
            Rc::new(FlaggedProbeNoticeAdjuster {
                flags: flags.clone(),
                flag: "probe-notice-default".to_string(),
                percent: 1.7,
            }),
            Rc::new(PercentProbeNoticeAdjuster { percent: 24.4 }),
            Rc::new(MinimumProbeNoticeAdjuster { minimum: 50.0 }),
            Rc::new(LabelProbeNoticeAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.36), ("online".to_string(), 0.73)]),
            }),
            Rc::new(CapProbeNoticeAdjuster { cap: 4240.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 70265.2,
    }
}
