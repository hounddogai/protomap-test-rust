use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the suspension score calculation.
pub trait SuspensionScoreModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSuspensionScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SuspensionScoreModifier for FlaggedSuspensionScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SuspensionScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSuspensionScoreAdjuster {
    pub cap: f64,
}

impl SuspensionScoreModifier for CapSuspensionScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SuspensionScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionScoreAdjuster {
    pub percent: f64,
}

impl SuspensionScoreModifier for PercentSuspensionScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeSuspensionScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SuspensionScoreModifier for ScopeSuspensionScoreAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &SuspensionScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionScoreAdjuster {
    pub minimum: f64,
}

impl SuspensionScoreModifier for MinimumSuspensionScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionScoreModifier for SuspensionScoreDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionScore, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

/// Runs the suspension score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionScoreEngine {
    pub steps: Vec<Rc<dyn SuspensionScoreModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionScoreEngine {
    pub fn evaluate(&mut self, item: &SuspensionScore) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SuspensionScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_suspension_score_engine(flags: &flags::Registry) -> SuspensionScoreEngine {
    SuspensionScoreEngine {
        steps: vec![
            Rc::new(FlaggedSuspensionScoreAdjuster {
                flags: flags.clone(),
                flag: "suspension-score-capped".to_string(),
                percent: -2.9,
            }) as Rc<dyn SuspensionScoreModifier>,
            Rc::new(CapSuspensionScoreAdjuster { cap: 48913.0 }),
            Rc::new(PercentSuspensionScoreAdjuster { percent: 17.5 }),
            Rc::new(ScopeSuspensionScoreAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.07)]),
            }),
            Rc::new(MinimumSuspensionScoreAdjuster { minimum: 12.7 }),
            Rc::new(SuspensionScoreDurationMinutesOffsetAdjuster { weight: 0.11 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 97289.9,
    }
}
