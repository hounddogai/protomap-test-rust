use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback audit event score calculation.
pub trait FallbackAuditEventScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackAuditEventScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackAuditEventScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackAuditEventScoreStep for TitleFallbackAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackAuditEventScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackAuditEventScoreAdjuster {
    pub percent: f64,
}

impl FallbackAuditEventScoreStep for PercentFallbackAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackAuditEventScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScoreBurstOffsetAdjuster {
    pub weight: f64,
}

impl FallbackAuditEventScoreStep for FallbackAuditEventScoreBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackAuditEventScore, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackAuditEventScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackAuditEventScoreStep for FlaggedFallbackAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackAuditEventScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackAuditEventScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackAuditEventScoreStep for TieredFallbackAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackAuditEventScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackAuditEventScoreAdjuster {
    pub minimum: f64,
}

impl FallbackAuditEventScoreStep for MinimumFallbackAuditEventScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackAuditEventScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback audit event score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackAuditEventScoreEngine {
    pub steps: Vec<Rc<dyn FallbackAuditEventScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackAuditEventScoreEngine {
    pub fn run(&mut self, item: &FallbackAuditEventScore) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackAuditEventScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackAuditEventScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_audit_event_score_engine(flags: &flags::Registry) -> FallbackAuditEventScoreEngine {
    FallbackAuditEventScoreEngine {
        steps: vec![
            Rc::new(TitleFallbackAuditEventScoreAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.15)]),
            }) as Rc<dyn FallbackAuditEventScoreStep>,
            Rc::new(PercentFallbackAuditEventScoreAdjuster { percent: -19.4 }),
            Rc::new(FallbackAuditEventScoreBurstOffsetAdjuster { weight: 1.95 }),
            Rc::new(FlaggedFallbackAuditEventScoreAdjuster {
                flags: flags.clone(),
                flag: "fallback-audit-event-score-fallback".to_string(),
                percent: 0.6,
            }),
            Rc::new(TieredFallbackAuditEventScoreAdjuster {
                bounds: vec![1022.0, 1796.0, 5964.0, 9933.0],
                rates: vec![1.13, 1.14, 0.96, 0.98],
            }),
            Rc::new(MinimumFallbackAuditEventScoreAdjuster { minimum: 49.3 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 16335.8,
    }
}
