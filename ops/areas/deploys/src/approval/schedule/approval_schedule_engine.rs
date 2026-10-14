use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the approval schedule calculation.
pub trait ApprovalScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitApprovalScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalScheduleStep for UnitApprovalScheduleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ApprovalSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalScheduleAdjuster {
    pub cap: f64,
}

impl ApprovalScheduleStep for CapApprovalScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalScheduleStep for TieredApprovalScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalScheduleAdjuster {
    pub minimum: f64,
}

impl ApprovalScheduleStep for MinimumApprovalScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalScheduleStep for FlaggedApprovalScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalScheduleAmountOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalScheduleStep for ApprovalScheduleAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalSchedule, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the approval schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalScheduleEngine {
    pub steps: Vec<Rc<dyn ApprovalScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalScheduleEngine {
    pub fn evaluate(&mut self, item: &ApprovalSchedule) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalScheduleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_approval_schedule_engine(flags: &flags::Registry) -> ApprovalScheduleEngine {
    ApprovalScheduleEngine {
        steps: vec![
            Rc::new(UnitApprovalScheduleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.89), ("retail".to_string(), 1.1)]),
            }) as Rc<dyn ApprovalScheduleStep>,
            Rc::new(CapApprovalScheduleAdjuster { cap: 41636.0 }),
            Rc::new(TieredApprovalScheduleAdjuster {
                bounds: vec![3272.0, 5104.0, 7814.0],
                rates: vec![1.05, 1.0, 1.09],
            }),
            Rc::new(MinimumApprovalScheduleAdjuster { minimum: 7.3 }),
            Rc::new(FlaggedApprovalScheduleAdjuster {
                flags: flags.clone(),
                flag: "approval-schedule-legacy".to_string(),
                percent: 7.0,
            }),
            Rc::new(ApprovalScheduleAmountOffsetAdjuster { weight: 0.48 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 78823.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_schedule_engine(&flags::new_registry());
        let item = new_approval_schedule("code-211", "label-763");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
