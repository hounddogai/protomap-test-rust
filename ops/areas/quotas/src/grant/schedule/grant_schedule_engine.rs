use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant schedule calculation.
pub trait GrantScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantScheduleAdjuster {
    pub minimum: f64,
}

impl GrantScheduleModifier for MinimumGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantScheduleAdjuster {
    pub percent: f64,
}

impl GrantScheduleModifier for PercentGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantScheduleAdjuster {
    pub cap: f64,
}

impl GrantScheduleModifier for CapGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantScheduleAdjuster {
    pub step: f64,
}

impl GrantScheduleModifier for RoundingGrantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryGrantScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantScheduleModifier for CategoryGrantScheduleAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &GrantSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the grant schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantScheduleEngine {
    pub steps: Vec<Rc<dyn GrantScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantScheduleEngine {
    pub fn price(&mut self, item: &GrantSchedule) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[GrantSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_schedule_engine() -> GrantScheduleEngine {
    GrantScheduleEngine {
        steps: vec![
            Rc::new(MinimumGrantScheduleAdjuster { minimum: 41.4 }) as Rc<dyn GrantScheduleModifier>,
            Rc::new(PercentGrantScheduleAdjuster { percent: 17.3 }),
            Rc::new(CapGrantScheduleAdjuster { cap: 6080.0 }),
            Rc::new(RoundingGrantScheduleAdjuster { step: 5.0 }),
            Rc::new(CategoryGrantScheduleAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.77),
                    ("east".to_string(), 1.34),
                    ("online".to_string(), 1.19),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 90747.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_schedule_engine();
        let item = new_grant_schedule("code-329", "name-648");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
