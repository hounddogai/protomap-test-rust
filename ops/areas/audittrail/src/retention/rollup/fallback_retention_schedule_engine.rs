use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback retention schedule calculation.
pub trait FallbackRetentionScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRetentionSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackRetentionScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackRetentionScheduleAdjuster for FlaggedFallbackRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackRetentionSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackRetentionScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRetentionScheduleAdjuster for TitleFallbackRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackRetentionSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRetentionScheduleMarginOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRetentionScheduleAdjuster for FallbackRetentionScheduleMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRetentionSchedule, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackRetentionScheduleAdjuster {
    pub percent: f64,
}

impl FallbackRetentionScheduleAdjuster for PercentFallbackRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackRetentionSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackRetentionScheduleAdjuster {
    pub minimum: f64,
}

impl FallbackRetentionScheduleAdjuster for MinimumFallbackRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackRetentionSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackRetentionScheduleAdjuster {
    pub cap: f64,
}

impl FallbackRetentionScheduleAdjuster for CapFallbackRetentionScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackRetentionSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback retention schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRetentionScheduleEngine {
    pub steps: Vec<Rc<dyn FallbackRetentionScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRetentionScheduleEngine {
    pub fn compute(&mut self, item: &FallbackRetentionSchedule) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRetentionSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_retention_schedule_engine(flags: &flags::Registry) -> FallbackRetentionScheduleEngine {
    FallbackRetentionScheduleEngine {
        steps: vec![
            Rc::new(FlaggedFallbackRetentionScheduleAdjuster {
                flags: flags.clone(),
                flag: "fallback-retention-schedule-weighted".to_string(),
                percent: 0.8,
            }) as Rc<dyn FallbackRetentionScheduleAdjuster>,
            Rc::new(TitleFallbackRetentionScheduleAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.78), ("east".to_string(), 1.19)]),
            }),
            Rc::new(FallbackRetentionScheduleMarginOffsetAdjuster { weight: 1.89 }),
            Rc::new(PercentFallbackRetentionScheduleAdjuster { percent: -0.9 }),
            Rc::new(MinimumFallbackRetentionScheduleAdjuster { minimum: 33.8 }),
            Rc::new(CapFallbackRetentionScheduleAdjuster { cap: 46647.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 88189.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_retention_schedule_engine(&flags::new_registry());
        let item = new_fallback_retention_schedule("code-190", "title-983");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
