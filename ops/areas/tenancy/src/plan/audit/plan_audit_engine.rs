use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan audit calculation.
pub trait PlanAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanAuditAdjuster {
    pub cap: f64,
}

impl PlanAuditFactor for CapPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePlanAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanAuditFactor for NamePlanAuditAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PlanAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanAuditAdjuster {
    pub minimum: f64,
}

impl PlanAuditFactor for MinimumPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanAuditFactor for FlaggedPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanAuditAdjuster {
    pub percent: f64,
}

impl PlanAuditFactor for PercentPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanAuditAdjuster {
    pub step: f64,
}

impl PlanAuditFactor for RoundingPlanAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the plan audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanAuditEngine {
    pub steps: Vec<Rc<dyn PlanAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanAuditEngine {
    pub fn evaluate(&mut self, item: &PlanAudit) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[PlanAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_audit_engine(flags: &flags::Registry) -> PlanAuditEngine {
    PlanAuditEngine {
        steps: vec![
            Rc::new(CapPlanAuditAdjuster { cap: 2665.0 }) as Rc<dyn PlanAuditFactor>,
            Rc::new(NamePlanAuditAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.34),
                    ("online".to_string(), 0.82),
                    ("central".to_string(), 1.13),
                ]),
            }),
            Rc::new(MinimumPlanAuditAdjuster { minimum: 47.3 }),
            Rc::new(FlaggedPlanAuditAdjuster {
                flags: flags.clone(),
                flag: "plan-audit-scheduled".to_string(),
                percent: 3.8,
            }),
            Rc::new(PercentPlanAuditAdjuster { percent: 13.3 }),
            Rc::new(RoundingPlanAuditAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 20073.3,
    }
}
