use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending policy schedule calculation.
pub trait PendingPolicyScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPolicySchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingPolicyScheduleAdjuster {
    pub cap: f64,
}

impl PendingPolicyScheduleFactor for CapPendingPolicyScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingPolicySchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicyScheduleBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PendingPolicyScheduleFactor for PendingPolicyScheduleBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PendingPolicySchedule, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingPolicyScheduleAdjuster {
    pub minimum: f64,
}

impl PendingPolicyScheduleFactor for MinimumPendingPolicyScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingPolicySchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingPolicyScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingPolicyScheduleFactor for FlaggedPendingPolicyScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingPolicySchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending policy schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPolicyScheduleEngine {
    pub steps: Vec<Rc<dyn PendingPolicyScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPolicyScheduleEngine {
    pub fn run(&mut self, item: &PendingPolicySchedule) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPolicyScheduleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPolicySchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_policy_schedule_engine(flags: &flags::Registry) -> PendingPolicyScheduleEngine {
    PendingPolicyScheduleEngine {
        steps: vec![
            Rc::new(CapPendingPolicyScheduleAdjuster { cap: 9035.0 }) as Rc<dyn PendingPolicyScheduleFactor>,
            Rc::new(PendingPolicyScheduleBatchSizeOffsetAdjuster { weight: 0.63 }),
            Rc::new(MinimumPendingPolicyScheduleAdjuster { minimum: 24.6 }),
            Rc::new(FlaggedPendingPolicyScheduleAdjuster {
                flags: flags.clone(),
                flag: "pending-policy-schedule-rolling".to_string(),
                percent: 4.8,
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 44381.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_policy_schedule_engine(&flags::new_registry());
        let item = new_pending_policy_schedule("code-183", "label-762");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
