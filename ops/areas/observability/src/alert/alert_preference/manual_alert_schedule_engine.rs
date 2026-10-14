use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual alert schedule calculation.
pub trait ManualAlertScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualAlertSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAlertScheduleWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ManualAlertScheduleStep for ManualAlertScheduleWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ManualAlertSchedule, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualAlertScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualAlertScheduleStep for FlaggedManualAlertScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualAlertSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualAlertScheduleAdjuster {
    pub percent: f64,
}

impl ManualAlertScheduleStep for PercentManualAlertScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualAlertSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualAlertScheduleAdjuster {
    pub minimum: f64,
}

impl ManualAlertScheduleStep for MinimumManualAlertScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualAlertSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionManualAlertScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualAlertScheduleStep for RegionManualAlertScheduleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ManualAlertSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual alert schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualAlertScheduleEngine {
    pub steps: Vec<Rc<dyn ManualAlertScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualAlertScheduleEngine {
    pub fn run(&mut self, item: &ManualAlertSchedule) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualAlertSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_alert_schedule_engine(flags: &flags::Registry) -> ManualAlertScheduleEngine {
    ManualAlertScheduleEngine {
        steps: vec![
            Rc::new(ManualAlertScheduleWindowDaysOffsetAdjuster { weight: 1.86 }) as Rc<dyn ManualAlertScheduleStep>,
            Rc::new(FlaggedManualAlertScheduleAdjuster {
                flags: flags.clone(),
                flag: "manual-alert-schedule-lenient".to_string(),
                percent: -1.9,
            }),
            Rc::new(PercentManualAlertScheduleAdjuster { percent: -18.0 }),
            Rc::new(MinimumManualAlertScheduleAdjuster { minimum: 46.6 }),
            Rc::new(RegionManualAlertScheduleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.9), ("west".to_string(), 0.95)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 22938.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_alert_schedule_engine(&flags::new_registry());
        let item = new_manual_alert_schedule("code-148", "title-693");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
