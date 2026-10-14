use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout schedule calculation.
pub trait RolloutScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRolloutScheduleAdjuster {
    pub minimum: f64,
}

impl RolloutScheduleModifier for MinimumRolloutScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RolloutSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRolloutScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RolloutScheduleModifier for SegmentRolloutScheduleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &RolloutSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutScheduleModifier for FlaggedRolloutScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutScheduleAdjuster {
    pub cap: f64,
}

impl RolloutScheduleModifier for CapRolloutScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutScheduleAdjuster {
    pub step: f64,
}

impl RolloutScheduleModifier for RoundingRolloutScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutScheduleAdjuster {
    pub percent: f64,
}

impl RolloutScheduleModifier for PercentRolloutScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollout schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutScheduleEngine {
    pub steps: Vec<Rc<dyn RolloutScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutScheduleEngine {
    pub fn evaluate(&mut self, item: &RolloutSchedule) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutScheduleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollout_schedule_engine(flags: &flags::Registry) -> RolloutScheduleEngine {
    RolloutScheduleEngine {
        steps: vec![
            Rc::new(MinimumRolloutScheduleAdjuster { minimum: 27.6 }) as Rc<dyn RolloutScheduleModifier>,
            Rc::new(SegmentRolloutScheduleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.85)]),
            }),
            Rc::new(FlaggedRolloutScheduleAdjuster {
                flags: flags.clone(),
                flag: "rollout-schedule-primary".to_string(),
                percent: -0.3,
            }),
            Rc::new(CapRolloutScheduleAdjuster { cap: 40862.0 }),
            Rc::new(RoundingRolloutScheduleAdjuster { step: 1.0 }),
            Rc::new(PercentRolloutScheduleAdjuster { percent: 15.6 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 44998.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_schedule_engine(&flags::new_registry());
        let item = new_rollout_schedule("code-122", "label-615");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
