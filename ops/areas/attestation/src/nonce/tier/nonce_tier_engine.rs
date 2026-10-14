use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the nonce tier calculation.
pub trait NonceTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceTierTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl NonceTierStep for NonceTierTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &NonceTier, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceTierStep for TieredNonceTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceTierStep for FlaggedNonceTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelNonceTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NonceTierStep for LabelNonceTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &NonceTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceTierAdjuster {
    pub percent: f64,
}

impl NonceTierStep for PercentNonceTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the nonce tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceTierEngine {
    pub steps: Vec<Rc<dyn NonceTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceTierEngine {
    pub fn evaluate(&mut self, item: &NonceTier) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceTierStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_tier_engine(flags: &flags::Registry) -> NonceTierEngine {
    NonceTierEngine {
        steps: vec![
            Rc::new(NonceTierTtlSecondsOffsetAdjuster { weight: 1.63 }) as Rc<dyn NonceTierStep>,
            Rc::new(TieredNonceTierAdjuster {
                bounds: vec![1720.0, 4371.0],
                rates: vec![1.07, 0.82],
            }),
            Rc::new(FlaggedNonceTierAdjuster {
                flags: flags.clone(),
                flag: "nonce-tier-weighted".to_string(),
                percent: 4.5,
            }),
            Rc::new(LabelNonceTierAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.06),
                    ("central".to_string(), 0.9),
                    ("east".to_string(), 1.09),
                ]),
            }),
            Rc::new(PercentNonceTierAdjuster { percent: -6.5 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 53968.8,
    }
}
