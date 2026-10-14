use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan schedule calculation.
pub trait PlanScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanScheduleAdjuster {
    pub minimum: f64,
}

impl PlanScheduleStep for MinimumPlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanScheduleAdjuster {
    pub percent: f64,
}

impl PlanScheduleStep for PercentPlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanScheduleStep for TieredPlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePlanScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanScheduleStep for NamePlanScheduleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PlanSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanScheduleRatioOffsetAdjuster {
    pub weight: f64,
}

impl PlanScheduleStep for PlanScheduleRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &PlanSchedule, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

/// Runs the plan schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanScheduleEngine {
    pub steps: Vec<Rc<dyn PlanScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanScheduleEngine {
    pub fn run(&mut self, item: &PlanSchedule) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PlanSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_plan_schedule_engine() -> PlanScheduleEngine {
    PlanScheduleEngine {
        steps: vec![
            Rc::new(MinimumPlanScheduleAdjuster { minimum: 38.8 }) as Rc<dyn PlanScheduleStep>,
            Rc::new(PercentPlanScheduleAdjuster { percent: -14.7 }),
            Rc::new(TieredPlanScheduleAdjuster {
                bounds: vec![578.0, 1351.0, 5674.0, 7678.0],
                rates: vec![0.85, 0.93, 1.02, 1.17],
            }),
            Rc::new(NamePlanScheduleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.11), ("south".to_string(), 1.16)]),
            }),
            Rc::new(PlanScheduleRatioOffsetAdjuster { weight: 0.74 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 26041.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_schedule_engine();
        let item = new_plan_schedule("code-171", "name-949");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
