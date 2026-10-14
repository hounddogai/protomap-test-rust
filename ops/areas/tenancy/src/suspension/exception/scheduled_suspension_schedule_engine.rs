use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled suspension schedule calculation.
pub trait ScheduledSuspensionScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSuspensionSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledSuspensionScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledSuspensionScheduleFactor for FlaggedScheduledSuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledSuspensionSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledSuspensionScheduleAdjuster {
    pub step: f64,
}

impl ScheduledSuspensionScheduleFactor for RoundingScheduledSuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledSuspensionSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledSuspensionScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledSuspensionScheduleFactor for TieredScheduledSuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledSuspensionSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScheduledSuspensionScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledSuspensionScheduleFactor for SegmentScheduledSuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ScheduledSuspensionSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledSuspensionScheduleAdjuster {
    pub cap: f64,
}

impl ScheduledSuspensionScheduleFactor for CapScheduledSuspensionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledSuspensionSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the scheduled suspension schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionScheduleEngine {
    pub steps: Vec<Rc<dyn ScheduledSuspensionScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSuspensionScheduleEngine {
    pub fn price(&mut self, item: &ScheduledSuspensionSchedule) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledSuspensionScheduleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSuspensionSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_suspension_schedule_engine(flags: &flags::Registry) -> ScheduledSuspensionScheduleEngine {
    ScheduledSuspensionScheduleEngine {
        steps: vec![
            Rc::new(FlaggedScheduledSuspensionScheduleAdjuster {
                flags: flags.clone(),
                flag: "scheduled-suspension-schedule-scheduled".to_string(),
                percent: 8.2,
            }) as Rc<dyn ScheduledSuspensionScheduleFactor>,
            Rc::new(RoundingScheduledSuspensionScheduleAdjuster { step: 0.1 }),
            Rc::new(TieredScheduledSuspensionScheduleAdjuster {
                bounds: vec![7042.0, 7928.0],
                rates: vec![1.18, 1.18],
            }),
            Rc::new(SegmentScheduledSuspensionScheduleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.99), ("retail".to_string(), 1.06)]),
            }),
            Rc::new(CapScheduledSuspensionScheduleAdjuster { cap: 38450.0 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 85430.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_suspension_schedule_engine(&flags::new_registry());
        let item = new_scheduled_suspension_schedule("code-838", "label-59");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
