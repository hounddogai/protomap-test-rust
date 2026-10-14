use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling approval summary calculation.
pub trait RollingApprovalSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingApprovalSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingApprovalSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingApprovalSummaryFactor for FlaggedRollingApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingApprovalSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalSummaryMarginOffsetAdjuster {
    pub weight: f64,
}

impl RollingApprovalSummaryFactor for RollingApprovalSummaryMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RollingApprovalSummary, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRollingApprovalSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingApprovalSummaryFactor for UnitRollingApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RollingApprovalSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingApprovalSummaryAdjuster {
    pub step: f64,
}

impl RollingApprovalSummaryFactor for RoundingRollingApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingApprovalSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingApprovalSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingApprovalSummaryFactor for TieredRollingApprovalSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingApprovalSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling approval summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingApprovalSummaryEngine {
    pub steps: Vec<Rc<dyn RollingApprovalSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingApprovalSummaryEngine {
    pub fn price(&mut self, item: &RollingApprovalSummary) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingApprovalSummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingApprovalSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_approval_summary_engine(flags: &flags::Registry) -> RollingApprovalSummaryEngine {
    RollingApprovalSummaryEngine {
        steps: vec![
            Rc::new(FlaggedRollingApprovalSummaryAdjuster {
                flags: flags.clone(),
                flag: "rolling-approval-summary-tiered".to_string(),
                percent: 9.3,
            }) as Rc<dyn RollingApprovalSummaryFactor>,
            Rc::new(RollingApprovalSummaryMarginOffsetAdjuster { weight: 0.92 }),
            Rc::new(UnitRollingApprovalSummaryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.26), ("south".to_string(), 1.08)]),
            }),
            Rc::new(RoundingRollingApprovalSummaryAdjuster { step: 0.5 }),
            Rc::new(TieredRollingApprovalSummaryAdjuster {
                bounds: vec![187.0, 5915.0, 7334.0, 9566.0],
                rates: vec![1.0, 1.08, 0.85, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 4121.1,
    }
}
