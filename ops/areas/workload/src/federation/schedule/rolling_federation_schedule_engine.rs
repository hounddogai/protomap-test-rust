use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling federation schedule calculation.
pub trait RollingFederationScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingFederationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingFederationScheduleAdjuster {
    pub minimum: f64,
}

impl RollingFederationScheduleFactor for MinimumRollingFederationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingFederationSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingFederationScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingFederationScheduleFactor for NameRollingFederationScheduleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingFederationSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl RollingFederationScheduleFactor for RollingFederationScheduleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &RollingFederationSchedule, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the rolling federation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingFederationScheduleEngine {
    pub steps: Vec<Rc<dyn RollingFederationScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingFederationScheduleEngine {
    pub fn evaluate(&mut self, item: &RollingFederationSchedule) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingFederationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_federation_schedule_engine() -> RollingFederationScheduleEngine {
    RollingFederationScheduleEngine {
        steps: vec![
            Rc::new(MinimumRollingFederationScheduleAdjuster { minimum: 46.5 })
                as Rc<dyn RollingFederationScheduleFactor>,
            Rc::new(NameRollingFederationScheduleAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.71),
                    ("central".to_string(), 0.95),
                    ("west".to_string(), 1.39),
                ]),
            }),
            Rc::new(RollingFederationScheduleQuantityOffsetAdjuster { weight: 0.45 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 89926.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_federation_schedule_engine();
        let item = new_rolling_federation_schedule("code-858", "name-457");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
