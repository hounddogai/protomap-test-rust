use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary freeze schedule calculation.
pub trait PrimaryFreezeScheduleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryFreezeSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFreezeScheduleRatioOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryFreezeScheduleStep for PrimaryFreezeScheduleRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryFreezeSchedule, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryFreezeScheduleAdjuster {
    pub minimum: f64,
}

impl PrimaryFreezeScheduleStep for MinimumPrimaryFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryFreezeSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryFreezeScheduleAdjuster {
    pub step: f64,
}

impl PrimaryFreezeScheduleStep for RoundingPrimaryFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryFreezeSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryFreezeScheduleAdjuster {
    pub percent: f64,
}

impl PrimaryFreezeScheduleStep for PercentPrimaryFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryFreezeSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryFreezeScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryFreezeScheduleStep for TieredPrimaryFreezeScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryFreezeSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the primary freeze schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryFreezeScheduleEngine {
    pub steps: Vec<Rc<dyn PrimaryFreezeScheduleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryFreezeScheduleEngine {
    pub fn price(&mut self, item: &PrimaryFreezeSchedule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryFreezeScheduleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryFreezeSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_freeze_schedule_engine() -> PrimaryFreezeScheduleEngine {
    PrimaryFreezeScheduleEngine {
        steps: vec![
            Rc::new(PrimaryFreezeScheduleRatioOffsetAdjuster { weight: 1.47 }) as Rc<dyn PrimaryFreezeScheduleStep>,
            Rc::new(MinimumPrimaryFreezeScheduleAdjuster { minimum: 28.8 }),
            Rc::new(RoundingPrimaryFreezeScheduleAdjuster { step: 0.25 }),
            Rc::new(PercentPrimaryFreezeScheduleAdjuster { percent: 17.4 }),
            Rc::new(TieredPrimaryFreezeScheduleAdjuster {
                bounds: vec![4417.0, 8670.0],
                rates: vec![0.82, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 87069.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_freeze_schedule_engine();
        let item = new_primary_freeze_schedule("code-983", "label-851");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
