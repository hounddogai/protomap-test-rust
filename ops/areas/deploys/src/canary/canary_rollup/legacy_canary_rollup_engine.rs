use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy canary rollup calculation.
pub trait LegacyCanaryRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyCanaryRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyCanaryRollupAdjuster {
    pub minimum: f64,
}

impl LegacyCanaryRollupFactor for MinimumLegacyCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyCanaryRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyCanaryRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyCanaryRollupFactor for TieredLegacyCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyCanaryRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryRollupUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyCanaryRollupFactor for LegacyCanaryRollupUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyCanaryRollup, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyCanaryRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyCanaryRollupFactor for FlaggedLegacyCanaryRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyCanaryRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy canary rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyCanaryRollupEngine {
    pub steps: Vec<Rc<dyn LegacyCanaryRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyCanaryRollupEngine {
    pub fn evaluate(&mut self, item: &LegacyCanaryRollup) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyCanaryRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_canary_rollup_engine(flags: &flags::Registry) -> LegacyCanaryRollupEngine {
    LegacyCanaryRollupEngine {
        steps: vec![
            Rc::new(MinimumLegacyCanaryRollupAdjuster { minimum: 3.7 }) as Rc<dyn LegacyCanaryRollupFactor>,
            Rc::new(TieredLegacyCanaryRollupAdjuster {
                bounds: vec![5006.0, 8254.0, 8333.0],
                rates: vec![0.9, 0.93, 0.93],
            }),
            Rc::new(LegacyCanaryRollupUpdatedAtOffsetAdjuster { weight: 1.76 }),
            Rc::new(FlaggedLegacyCanaryRollupAdjuster {
                flags: flags.clone(),
                flag: "legacy-canary-rollup-default".to_string(),
                percent: -1.3,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 99824.3,
    }
}
