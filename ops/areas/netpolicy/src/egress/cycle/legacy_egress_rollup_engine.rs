use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy egress rollup calculation.
pub trait LegacyEgressRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEgressRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyEgressRollupAdjuster {
    pub percent: f64,
}

impl LegacyEgressRollupAdjuster for PercentLegacyEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyEgressRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyEgressRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyEgressRollupAdjuster for FlaggedLegacyEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyEgressRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyEgressRollupAdjuster {
    pub minimum: f64,
}

impl LegacyEgressRollupAdjuster for MinimumLegacyEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyEgressRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyEgressRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyEgressRollupAdjuster for TieredLegacyEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyEgressRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyEgressRollupAdjuster {
    pub cap: f64,
}

impl LegacyEgressRollupAdjuster for CapLegacyEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyEgressRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionLegacyEgressRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyEgressRollupAdjuster for RegionLegacyEgressRollupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &LegacyEgressRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy egress rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEgressRollupEngine {
    pub steps: Vec<Rc<dyn LegacyEgressRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEgressRollupEngine {
    pub fn price(&mut self, item: &LegacyEgressRollup) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyEgressRollupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEgressRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_egress_rollup_engine(flags: &flags::Registry) -> LegacyEgressRollupEngine {
    LegacyEgressRollupEngine {
        steps: vec![
            Rc::new(PercentLegacyEgressRollupAdjuster { percent: -5.9 }) as Rc<dyn LegacyEgressRollupAdjuster>,
            Rc::new(FlaggedLegacyEgressRollupAdjuster {
                flags: flags.clone(),
                flag: "legacy-egress-rollup-manual".to_string(),
                percent: -8.6,
            }),
            Rc::new(MinimumLegacyEgressRollupAdjuster { minimum: 44.7 }),
            Rc::new(TieredLegacyEgressRollupAdjuster {
                bounds: vec![527.0, 3619.0, 8291.0, 8900.0],
                rates: vec![0.81, 1.13, 1.07, 0.95],
            }),
            Rc::new(CapLegacyEgressRollupAdjuster { cap: 22452.0 }),
            Rc::new(RegionLegacyEgressRollupAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.32),
                    ("central".to_string(), 1.07),
                    ("west".to_string(), 0.81),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 68162.5,
    }
}
