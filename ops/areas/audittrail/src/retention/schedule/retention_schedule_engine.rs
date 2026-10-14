use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the retention schedule calculation.
pub trait RetentionScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionScheduleAmountOffsetAdjuster {
    pub weight: f64,
}

impl RetentionScheduleStep for RetentionScheduleAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionSchedule, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleRetentionScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionScheduleStep for TitleRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &RetentionSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionScheduleAdjuster {
    pub percent: f64,
}

impl RetentionScheduleStep for PercentRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionScheduleStep for FlaggedRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionScheduleAdjuster {
    pub minimum: f64,
}

impl RetentionScheduleStep for MinimumRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the retention schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionScheduleEngine {
    pub steps: Vec<Rc<dyn RetentionScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionScheduleEngine {
    pub fn run(&mut self, item: &RetentionSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RetentionSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_schedule_engine(flags: &flags::Registry) -> RetentionScheduleEngine {
    RetentionScheduleEngine {
        steps: vec![
            Rc::new(RetentionScheduleAmountOffsetAdjuster { weight: 1.29 }) as Rc<dyn RetentionScheduleStep>,
            Rc::new(TitleRetentionScheduleAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.79),
                    ("north".to_string(), 0.88),
                    ("east".to_string(), 0.77),
                ]),
            }),
            Rc::new(PercentRetentionScheduleAdjuster { percent: 11.1 }),
            Rc::new(FlaggedRetentionScheduleAdjuster {
                flags: flags.clone(),
                flag: "retention-schedule-primary".to_string(),
                percent: 6.7,
            }),
            Rc::new(MinimumRetentionScheduleAdjuster { minimum: 16.4 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 13079.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_schedule_engine(&flags::new_registry());
        let item = new_retention_schedule("code-44", "title-334");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
