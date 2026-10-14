use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled budget burn notice calculation.
pub trait ScheduledBudgetBurnNoticeModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledBudgetBurnNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBudgetBurnNoticeScoreOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledBudgetBurnNoticeModifier for ScheduledBudgetBurnNoticeScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledBudgetBurnNotice, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledBudgetBurnNoticeAdjuster {
    pub step: f64,
}

impl ScheduledBudgetBurnNoticeModifier for RoundingScheduledBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledBudgetBurnNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledBudgetBurnNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledBudgetBurnNoticeModifier for FlaggedScheduledBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledBudgetBurnNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledBudgetBurnNoticeAdjuster {
    pub percent: f64,
}

impl ScheduledBudgetBurnNoticeModifier for PercentScheduledBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledBudgetBurnNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionScheduledBudgetBurnNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledBudgetBurnNoticeModifier for RegionScheduledBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ScheduledBudgetBurnNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledBudgetBurnNoticeAdjuster {
    pub minimum: f64,
}

impl ScheduledBudgetBurnNoticeModifier for MinimumScheduledBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledBudgetBurnNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled budget burn notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledBudgetBurnNoticeEngine {
    pub steps: Vec<Rc<dyn ScheduledBudgetBurnNoticeModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledBudgetBurnNoticeEngine {
    pub fn run(&mut self, item: &ScheduledBudgetBurnNotice) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledBudgetBurnNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_budget_burn_notice_engine(flags: &flags::Registry) -> ScheduledBudgetBurnNoticeEngine {
    ScheduledBudgetBurnNoticeEngine {
        steps: vec![
            Rc::new(ScheduledBudgetBurnNoticeScoreOffsetAdjuster { weight: 1.98 })
                as Rc<dyn ScheduledBudgetBurnNoticeModifier>,
            Rc::new(RoundingScheduledBudgetBurnNoticeAdjuster { step: 0.25 }),
            Rc::new(FlaggedScheduledBudgetBurnNoticeAdjuster {
                flags: flags.clone(),
                flag: "scheduled-budget-burn-notice-scheduled".to_string(),
                percent: 2.3,
            }),
            Rc::new(PercentScheduledBudgetBurnNoticeAdjuster { percent: 0.1 }),
            Rc::new(RegionScheduledBudgetBurnNoticeAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.06)]),
            }),
            Rc::new(MinimumScheduledBudgetBurnNoticeAdjuster { minimum: 1.3 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 28292.9,
    }
}
