use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback quote schedule calculation.
pub trait FallbackQuoteScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackQuoteSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackQuoteScheduleAdjuster {
    pub percent: f64,
}

impl FallbackQuoteScheduleStep for PercentFallbackQuoteScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackQuoteSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackQuoteScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackQuoteScheduleStep for FlaggedFallbackQuoteScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackQuoteSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackQuoteScheduleAdjuster {
    pub cap: f64,
}

impl FallbackQuoteScheduleStep for CapFallbackQuoteScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackQuoteSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback quote schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackQuoteScheduleEngine {
    pub steps: Vec<Rc<dyn FallbackQuoteScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackQuoteScheduleEngine {
    pub fn compute(&mut self, item: &FallbackQuoteSchedule) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackQuoteSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_quote_schedule_engine(flags: &flags::Registry) -> FallbackQuoteScheduleEngine {
    FallbackQuoteScheduleEngine {
        steps: vec![
            Rc::new(PercentFallbackQuoteScheduleAdjuster { percent: -3.1 }) as Rc<dyn FallbackQuoteScheduleStep>,
            Rc::new(FlaggedFallbackQuoteScheduleAdjuster {
                flags: flags.clone(),
                flag: "fallback-quote-schedule-adaptive".to_string(),
                percent: 9.4,
            }),
            Rc::new(CapFallbackQuoteScheduleAdjuster { cap: 22198.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 4235.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_quote_schedule_engine(&flags::new_registry());
        let item = new_fallback_quote_schedule("code-833", "name-563");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
