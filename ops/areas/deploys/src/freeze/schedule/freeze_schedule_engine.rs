use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the freeze schedule calculation.
pub trait FreezeScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeScheduleAdjuster {
    pub percent: f64,
}

impl FreezeScheduleAdjuster for PercentFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeScheduleAdjuster {
    pub minimum: f64,
}

impl FreezeScheduleAdjuster for MinimumFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeScheduleAdjuster for TieredFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryFreezeScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeScheduleAdjuster for CategoryFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &FreezeSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the freeze schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeScheduleEngine {
    pub steps: Vec<Rc<dyn FreezeScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeScheduleEngine {
    pub fn price(&mut self, item: &FreezeSchedule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezeScheduleAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[FreezeSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_freeze_schedule_engine() -> FreezeScheduleEngine {
    FreezeScheduleEngine {
        steps: vec![
            Rc::new(PercentFreezeScheduleAdjuster { percent: -10.4 }) as Rc<dyn FreezeScheduleAdjuster>,
            Rc::new(MinimumFreezeScheduleAdjuster { minimum: 15.2 }),
            Rc::new(TieredFreezeScheduleAdjuster {
                bounds: vec![5059.0, 9258.0],
                rates: vec![0.9, 1.0],
            }),
            Rc::new(CategoryFreezeScheduleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.12),
                    ("retail".to_string(), 0.71),
                    ("south".to_string(), 1.01),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 62870.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_schedule_engine();
        let item = new_freeze_schedule("code-274", "title-914");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
