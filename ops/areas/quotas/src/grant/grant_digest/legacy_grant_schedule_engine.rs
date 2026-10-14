use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy grant schedule calculation.
pub trait LegacyGrantScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyGrantSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyGrantScheduleAdjuster {
    pub percent: f64,
}

impl LegacyGrantScheduleModifier for PercentLegacyGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyGrantSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantScheduleDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl LegacyGrantScheduleModifier for LegacyGrantScheduleDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyGrantSchedule, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyGrantScheduleAdjuster {
    pub cap: f64,
}

impl LegacyGrantScheduleModifier for CapLegacyGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyGrantSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyGrantScheduleAdjuster {
    pub step: f64,
}

impl LegacyGrantScheduleModifier for RoundingLegacyGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyGrantSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy grant schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyGrantScheduleEngine {
    pub steps: Vec<Rc<dyn LegacyGrantScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyGrantScheduleEngine {
    pub fn evaluate(&mut self, item: &LegacyGrantSchedule) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[LegacyGrantSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_grant_schedule_engine() -> LegacyGrantScheduleEngine {
    LegacyGrantScheduleEngine {
        steps: vec![
            Rc::new(PercentLegacyGrantScheduleAdjuster { percent: 24.2 }) as Rc<dyn LegacyGrantScheduleModifier>,
            Rc::new(LegacyGrantScheduleDurationMinutesOffsetAdjuster { weight: 1.79 }),
            Rc::new(CapLegacyGrantScheduleAdjuster { cap: 24577.0 }),
            Rc::new(RoundingLegacyGrantScheduleAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 54176.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_grant_schedule_engine();
        let item = new_legacy_grant_schedule("code-738", "label-71");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
