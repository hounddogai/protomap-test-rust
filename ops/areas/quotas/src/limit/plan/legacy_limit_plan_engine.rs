use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy limit plan calculation.
pub trait LegacyLimitPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyLimitPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlanWeightOffsetAdjuster {
    pub weight: f64,
}

impl LegacyLimitPlanFactor for LegacyLimitPlanWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyLimitPlan, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyLimitPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyLimitPlanFactor for FlaggedLegacyLimitPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyLimitPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameLegacyLimitPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyLimitPlanFactor for NameLegacyLimitPlanAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &LegacyLimitPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the legacy limit plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyLimitPlanEngine {
    pub steps: Vec<Rc<dyn LegacyLimitPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyLimitPlanEngine {
    pub fn price(&mut self, item: &LegacyLimitPlan) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyLimitPlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyLimitPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_limit_plan_engine(flags: &flags::Registry) -> LegacyLimitPlanEngine {
    LegacyLimitPlanEngine {
        steps: vec![
            Rc::new(LegacyLimitPlanWeightOffsetAdjuster { weight: 1.73 }) as Rc<dyn LegacyLimitPlanFactor>,
            Rc::new(FlaggedLegacyLimitPlanAdjuster {
                flags: flags.clone(),
                flag: "legacy-limit-plan-batched".to_string(),
                percent: 4.2,
            }),
            Rc::new(NameLegacyLimitPlanAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.77)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 72667.0,
    }
}
