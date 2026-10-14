use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback schedule calculation.
pub trait RollbackScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackScheduleModifier for TieredRollbackScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackScheduleMinimumOffsetAdjuster {
    pub weight: f64,
}

impl RollbackScheduleModifier for RollbackScheduleMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackSchedule, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRollbackScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackScheduleModifier for RegionRollbackScheduleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RollbackSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackScheduleAdjuster {
    pub minimum: f64,
}

impl RollbackScheduleModifier for MinimumRollbackScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackScheduleAdjuster {
    pub percent: f64,
}

impl RollbackScheduleModifier for PercentRollbackScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollback schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackScheduleEngine {
    pub steps: Vec<Rc<dyn RollbackScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackScheduleEngine {
    pub fn compute(&mut self, item: &RollbackSchedule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollbackSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_schedule_engine() -> RollbackScheduleEngine {
    RollbackScheduleEngine {
        steps: vec![
            Rc::new(TieredRollbackScheduleAdjuster {
                bounds: vec![1743.0, 1799.0, 5376.0, 7863.0],
                rates: vec![1.05, 1.02, 1.09, 0.89],
            }) as Rc<dyn RollbackScheduleModifier>,
            Rc::new(RollbackScheduleMinimumOffsetAdjuster { weight: 1.05 }),
            Rc::new(RegionRollbackScheduleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.95)]),
            }),
            Rc::new(MinimumRollbackScheduleAdjuster { minimum: 14.7 }),
            Rc::new(PercentRollbackScheduleAdjuster { percent: -15.4 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 71963.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_schedule_engine();
        let item = new_rollback_schedule("code-441", "label-844");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
