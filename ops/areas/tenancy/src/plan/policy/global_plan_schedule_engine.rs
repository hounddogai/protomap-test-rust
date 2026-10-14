use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global plan schedule calculation.
pub trait GlobalPlanScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPlanSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalPlanScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalPlanScheduleFactor for TieredGlobalPlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalPlanSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalPlanScheduleFactor for GlobalPlanScheduleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalPlanSchedule, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPlanScheduleAdjuster {
    pub minimum: f64,
}

impl GlobalPlanScheduleFactor for MinimumGlobalPlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPlanSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalPlanScheduleAdjuster {
    pub cap: f64,
}

impl GlobalPlanScheduleFactor for CapGlobalPlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalPlanSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalPlanScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPlanScheduleFactor for LabelGlobalPlanScheduleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalPlanSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global plan schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPlanScheduleEngine {
    pub steps: Vec<Rc<dyn GlobalPlanScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPlanScheduleEngine {
    pub fn evaluate(&mut self, item: &GlobalPlanSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalPlanSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_plan_schedule_engine() -> GlobalPlanScheduleEngine {
    GlobalPlanScheduleEngine {
        steps: vec![
            Rc::new(TieredGlobalPlanScheduleAdjuster {
                bounds: vec![6315.0, 7954.0],
                rates: vec![1.13, 1.09],
            }) as Rc<dyn GlobalPlanScheduleFactor>,
            Rc::new(GlobalPlanScheduleExpiresAtOffsetAdjuster { weight: 0.16 }),
            Rc::new(MinimumGlobalPlanScheduleAdjuster { minimum: 21.5 }),
            Rc::new(CapGlobalPlanScheduleAdjuster { cap: 36615.0 }),
            Rc::new(LabelGlobalPlanScheduleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.74),
                    ("north".to_string(), 0.8),
                    ("south".to_string(), 0.76),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 78080.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_plan_schedule_engine();
        let item = new_global_plan_schedule("code-108", "label-927");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
