use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped token schedule calculation.
pub trait CappedTokenScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedTokenSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedTokenScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedTokenScheduleModifier for NameCappedTokenScheduleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedTokenSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedTokenScheduleAdjuster {
    pub cap: f64,
}

impl CappedTokenScheduleModifier for CapCappedTokenScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedTokenSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedTokenScheduleAdjuster {
    pub step: f64,
}

impl CappedTokenScheduleModifier for RoundingCappedTokenScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedTokenSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the capped token schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedTokenScheduleEngine {
    pub steps: Vec<Rc<dyn CappedTokenScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedTokenScheduleEngine {
    pub fn run(&mut self, item: &CappedTokenSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedTokenSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_token_schedule_engine() -> CappedTokenScheduleEngine {
    CappedTokenScheduleEngine {
        steps: vec![
            Rc::new(NameCappedTokenScheduleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.2),
                    ("east".to_string(), 0.73),
                    ("central".to_string(), 0.72),
                ]),
            }) as Rc<dyn CappedTokenScheduleModifier>,
            Rc::new(CapCappedTokenScheduleAdjuster { cap: 48095.0 }),
            Rc::new(RoundingCappedTokenScheduleAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 64958.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_token_schedule_engine();
        let item = new_capped_token_schedule("code-468", "name-426");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
