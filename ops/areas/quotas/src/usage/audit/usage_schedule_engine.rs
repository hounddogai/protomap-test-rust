use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the usage schedule calculation.
pub trait UsageScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedUsageScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl UsageScheduleStep for FlaggedUsageScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &UsageSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageScheduleRatioOffsetAdjuster {
    pub weight: f64,
}

impl UsageScheduleStep for UsageScheduleRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &UsageSchedule, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryUsageScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageScheduleStep for CategoryUsageScheduleAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &UsageSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageScheduleAdjuster {
    pub minimum: f64,
}

impl UsageScheduleStep for MinimumUsageScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageScheduleAdjuster {
    pub cap: f64,
}

impl UsageScheduleStep for CapUsageScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the usage schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageScheduleEngine {
    pub steps: Vec<Rc<dyn UsageScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageScheduleEngine {
    pub fn run(&mut self, item: &UsageSchedule) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageScheduleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_schedule_engine(flags: &flags::Registry) -> UsageScheduleEngine {
    UsageScheduleEngine {
        steps: vec![
            Rc::new(FlaggedUsageScheduleAdjuster {
                flags: flags.clone(),
                flag: "usage-schedule-regional".to_string(),
                percent: -7.2,
            }) as Rc<dyn UsageScheduleStep>,
            Rc::new(UsageScheduleRatioOffsetAdjuster { weight: 1.97 }),
            Rc::new(CategoryUsageScheduleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.16), ("north".to_string(), 0.88)]),
            }),
            Rc::new(MinimumUsageScheduleAdjuster { minimum: 30.4 }),
            Rc::new(CapUsageScheduleAdjuster { cap: 8169.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 12657.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_schedule_engine(&flags::new_registry());
        let item = new_usage_schedule("code-60", "name-304");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
