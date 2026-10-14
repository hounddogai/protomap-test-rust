use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary budget burn score calculation.
pub trait PrimaryBudgetBurnScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryBudgetBurnScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryBudgetBurnScoreAdjuster {
    pub cap: f64,
}

impl PrimaryBudgetBurnScoreFactor for CapPrimaryBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryBudgetBurnScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryBudgetBurnScoreFactor for TieredPrimaryBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryBudgetBurnScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryBudgetBurnScoreFactor for FlaggedPrimaryBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPrimaryBudgetBurnScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryBudgetBurnScoreFactor for LabelPrimaryBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PrimaryBudgetBurnScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryBudgetBurnScoreAdjuster {
    pub percent: f64,
}

impl PrimaryBudgetBurnScoreFactor for PercentPrimaryBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnScoreFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryBudgetBurnScoreFactor for PrimaryBudgetBurnScoreFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnScore, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the primary budget burn score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryBudgetBurnScoreEngine {
    pub steps: Vec<Rc<dyn PrimaryBudgetBurnScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryBudgetBurnScoreEngine {
    pub fn run(&mut self, item: &PrimaryBudgetBurnScore) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryBudgetBurnScoreFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryBudgetBurnScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_budget_burn_score_engine(flags: &flags::Registry) -> PrimaryBudgetBurnScoreEngine {
    PrimaryBudgetBurnScoreEngine {
        steps: vec![
            Rc::new(CapPrimaryBudgetBurnScoreAdjuster { cap: 5928.0 }) as Rc<dyn PrimaryBudgetBurnScoreFactor>,
            Rc::new(TieredPrimaryBudgetBurnScoreAdjuster {
                bounds: vec![1287.0, 5216.0, 6865.0, 9712.0],
                rates: vec![1.08, 0.89, 0.88, 0.91],
            }),
            Rc::new(FlaggedPrimaryBudgetBurnScoreAdjuster {
                flags: flags.clone(),
                flag: "primary-budget-burn-score-strict".to_string(),
                percent: 9.4,
            }),
            Rc::new(LabelPrimaryBudgetBurnScoreAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.28),
                    ("south".to_string(), 0.75),
                    ("north".to_string(), 0.9),
                ]),
            }),
            Rc::new(PercentPrimaryBudgetBurnScoreAdjuster { percent: 7.0 }),
            Rc::new(PrimaryBudgetBurnScoreFloorValueOffsetAdjuster { weight: 0.44 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 22126.2,
    }
}
