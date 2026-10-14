use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual redaction rollup calculation.
pub trait ManualRedactionRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRedactionRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualRedactionRollupAdjuster {
    pub step: f64,
}

impl ManualRedactionRollupAdjuster for RoundingManualRedactionRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualRedactionRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualRedactionRollupAdjuster {
    pub minimum: f64,
}

impl ManualRedactionRollupAdjuster for MinimumManualRedactionRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualRedactionRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleManualRedactionRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRedactionRollupAdjuster for TitleManualRedactionRollupAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ManualRedactionRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualRedactionRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualRedactionRollupAdjuster for FlaggedManualRedactionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualRedactionRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionRollupRevisionOffsetAdjuster {
    pub weight: f64,
}

impl ManualRedactionRollupAdjuster for ManualRedactionRollupRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRedactionRollup, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualRedactionRollupAdjuster {
    pub percent: f64,
}

impl ManualRedactionRollupAdjuster for PercentManualRedactionRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualRedactionRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual redaction rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRedactionRollupEngine {
    pub steps: Vec<Rc<dyn ManualRedactionRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRedactionRollupEngine {
    pub fn run(&mut self, item: &ManualRedactionRollup) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[ManualRedactionRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_redaction_rollup_engine(flags: &flags::Registry) -> ManualRedactionRollupEngine {
    ManualRedactionRollupEngine {
        steps: vec![
            Rc::new(RoundingManualRedactionRollupAdjuster { step: 0.25 }) as Rc<dyn ManualRedactionRollupAdjuster>,
            Rc::new(MinimumManualRedactionRollupAdjuster { minimum: 11.0 }),
            Rc::new(TitleManualRedactionRollupAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.0),
                    ("retail".to_string(), 0.99),
                    ("online".to_string(), 0.87),
                ]),
            }),
            Rc::new(FlaggedManualRedactionRollupAdjuster {
                flags: flags.clone(),
                flag: "manual-redaction-rollup-legacy".to_string(),
                percent: -2.5,
            }),
            Rc::new(ManualRedactionRollupRevisionOffsetAdjuster { weight: 1.63 }),
            Rc::new(PercentManualRedactionRollupAdjuster { percent: -3.2 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 14468.3,
    }
}
