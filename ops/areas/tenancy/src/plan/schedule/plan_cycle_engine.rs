use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan cycle calculation.
pub trait PlanCycleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanCycleMinimumOffsetAdjuster {
    pub weight: f64,
}

impl PlanCycleAdjuster for PlanCycleMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &PlanCycle, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePlanCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanCycleAdjuster for SourcePlanCycleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PlanCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanCycleAdjuster {
    pub step: f64,
}

impl PlanCycleAdjuster for RoundingPlanCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanCycleAdjuster for FlaggedPlanCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanCycleEngine {
    pub steps: Vec<Rc<dyn PlanCycleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanCycleEngine {
    pub fn price(&mut self, item: &PlanCycle) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PlanCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_cycle_engine(flags: &flags::Registry) -> PlanCycleEngine {
    PlanCycleEngine {
        steps: vec![
            Rc::new(PlanCycleMinimumOffsetAdjuster { weight: 1.07 }) as Rc<dyn PlanCycleAdjuster>,
            Rc::new(SourcePlanCycleAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.37),
                    ("online".to_string(), 0.93),
                    ("north".to_string(), 0.72),
                ]),
            }),
            Rc::new(RoundingPlanCycleAdjuster { step: 5.0 }),
            Rc::new(FlaggedPlanCycleAdjuster {
                flags: flags.clone(),
                flag: "plan-cycle-global".to_string(),
                percent: 9.6,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 10826.9,
    }
}
