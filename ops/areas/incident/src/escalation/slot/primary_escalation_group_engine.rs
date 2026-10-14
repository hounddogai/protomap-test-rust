use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary escalation group calculation.
pub trait PrimaryEscalationGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEscalationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryEscalationGroupAdjuster {
    pub step: f64,
}

impl PrimaryEscalationGroupStep for RoundingPrimaryEscalationGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryEscalationGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryEscalationGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEscalationGroupStep for SourcePrimaryEscalationGroupAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryEscalationGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryEscalationGroupAdjuster {
    pub minimum: f64,
}

impl PrimaryEscalationGroupStep for MinimumPrimaryEscalationGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryEscalationGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryEscalationGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryEscalationGroupStep for TieredPrimaryEscalationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryEscalationGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryEscalationGroupAdjuster {
    pub percent: f64,
}

impl PrimaryEscalationGroupStep for PercentPrimaryEscalationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryEscalationGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryEscalationGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryEscalationGroupStep for FlaggedPrimaryEscalationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryEscalationGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary escalation group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationGroupEngine {
    pub steps: Vec<Rc<dyn PrimaryEscalationGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEscalationGroupEngine {
    pub fn price(&mut self, item: &PrimaryEscalationGroup) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryEscalationGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEscalationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_escalation_group_engine(flags: &flags::Registry) -> PrimaryEscalationGroupEngine {
    PrimaryEscalationGroupEngine {
        steps: vec![
            Rc::new(RoundingPrimaryEscalationGroupAdjuster { step: 0.5 }) as Rc<dyn PrimaryEscalationGroupStep>,
            Rc::new(SourcePrimaryEscalationGroupAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.39),
                    ("north".to_string(), 0.76),
                    ("retail".to_string(), 0.91),
                ]),
            }),
            Rc::new(MinimumPrimaryEscalationGroupAdjuster { minimum: 45.0 }),
            Rc::new(TieredPrimaryEscalationGroupAdjuster {
                bounds: vec![2781.0, 5181.0, 5311.0, 6872.0],
                rates: vec![0.94, 0.92, 0.84, 0.83],
            }),
            Rc::new(PercentPrimaryEscalationGroupAdjuster { percent: -1.4 }),
            Rc::new(FlaggedPrimaryEscalationGroupAdjuster {
                flags: flags.clone(),
                flag: "primary-escalation-group-legacy".to_string(),
                percent: 4.4,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 83799.3,
    }
}
