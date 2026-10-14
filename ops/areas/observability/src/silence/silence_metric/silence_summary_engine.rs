use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence summary calculation.
pub trait SilenceSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceSummaryAdjuster {
    pub step: f64,
}

impl SilenceSummaryModifier for RoundingSilenceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSilenceSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceSummaryModifier for LabelSilenceSummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SilenceSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceSummaryAdjuster {
    pub percent: f64,
}

impl SilenceSummaryModifier for PercentSilenceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceSummaryModifier for FlaggedSilenceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSilenceSummaryAdjuster {
    pub cap: f64,
}

impl SilenceSummaryModifier for CapSilenceSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SilenceSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSummaryBudgetOffsetAdjuster {
    pub weight: f64,
}

impl SilenceSummaryModifier for SilenceSummaryBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceSummary, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the silence summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceSummaryEngine {
    pub steps: Vec<Rc<dyn SilenceSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceSummaryEngine {
    pub fn run(&mut self, item: &SilenceSummary) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SilenceSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_summary_engine(flags: &flags::Registry) -> SilenceSummaryEngine {
    SilenceSummaryEngine {
        steps: vec![
            Rc::new(RoundingSilenceSummaryAdjuster { step: 0.1 }) as Rc<dyn SilenceSummaryModifier>,
            Rc::new(LabelSilenceSummaryAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.32),
                    ("retail".to_string(), 0.85),
                    ("west".to_string(), 1.09),
                ]),
            }),
            Rc::new(PercentSilenceSummaryAdjuster { percent: -15.6 }),
            Rc::new(FlaggedSilenceSummaryAdjuster {
                flags: flags.clone(),
                flag: "silence-summary-tiered".to_string(),
                percent: -9.8,
            }),
            Rc::new(CapSilenceSummaryAdjuster { cap: 42151.0 }),
            Rc::new(SilenceSummaryBudgetOffsetAdjuster { weight: 0.41 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 38422.3,
    }
}
