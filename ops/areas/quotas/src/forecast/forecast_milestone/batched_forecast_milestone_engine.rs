use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched forecast milestone calculation.
pub trait BatchedForecastMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedForecastMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedForecastMilestoneAdjuster {
    pub cap: f64,
}

impl BatchedForecastMilestoneAdjuster for CapBatchedForecastMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedForecastMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedForecastMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedForecastMilestoneAdjuster for FlaggedBatchedForecastMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedForecastMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedForecastMilestoneAdjuster {
    pub percent: f64,
}

impl BatchedForecastMilestoneAdjuster for PercentBatchedForecastMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedForecastMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedForecastMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedForecastMilestoneAdjuster for TieredBatchedForecastMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedForecastMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched forecast milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedForecastMilestoneEngine {
    pub steps: Vec<Rc<dyn BatchedForecastMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedForecastMilestoneEngine {
    pub fn price(&mut self, item: &BatchedForecastMilestone) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedForecastMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedForecastMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_forecast_milestone_engine(flags: &flags::Registry) -> BatchedForecastMilestoneEngine {
    BatchedForecastMilestoneEngine {
        steps: vec![
            Rc::new(CapBatchedForecastMilestoneAdjuster { cap: 47143.0 }) as Rc<dyn BatchedForecastMilestoneAdjuster>,
            Rc::new(FlaggedBatchedForecastMilestoneAdjuster {
                flags: flags.clone(),
                flag: "batched-forecast-milestone-rolling".to_string(),
                percent: -8.3,
            }),
            Rc::new(PercentBatchedForecastMilestoneAdjuster { percent: -18.8 }),
            Rc::new(TieredBatchedForecastMilestoneAdjuster {
                bounds: vec![6657.0, 9071.0],
                rates: vec![1.1, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 81343.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_forecast_milestone_engine(&flags::new_registry());
        let item = new_batched_forecast_milestone("code-870", "title-689");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
