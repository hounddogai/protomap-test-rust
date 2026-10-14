use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast assignment calculation.
pub trait ForecastAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastAssignmentAdjuster {
    pub step: f64,
}

impl ForecastAssignmentStep for RoundingForecastAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastAssignmentAdjuster {
    pub cap: f64,
}

impl ForecastAssignmentStep for CapForecastAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastAssignmentAdjuster {
    pub minimum: f64,
}

impl ForecastAssignmentStep for MinimumForecastAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastAssignmentStep for TieredForecastAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastAssignmentStep for FlaggedForecastAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the forecast assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastAssignmentEngine {
    pub steps: Vec<Rc<dyn ForecastAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastAssignmentEngine {
    pub fn evaluate(&mut self, item: &ForecastAssignment) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_forecast_assignment_engine(flags: &flags::Registry) -> ForecastAssignmentEngine {
    ForecastAssignmentEngine {
        steps: vec![
            Rc::new(RoundingForecastAssignmentAdjuster { step: 0.1 }) as Rc<dyn ForecastAssignmentStep>,
            Rc::new(CapForecastAssignmentAdjuster { cap: 38309.0 }),
            Rc::new(MinimumForecastAssignmentAdjuster { minimum: 48.5 }),
            Rc::new(TieredForecastAssignmentAdjuster {
                bounds: vec![1201.0, 2742.0, 5572.0],
                rates: vec![1.19, 0.87, 1.16],
            }),
            Rc::new(FlaggedForecastAssignmentAdjuster {
                flags: flags.clone(),
                flag: "forecast-assignment-default".to_string(),
                percent: -1.9,
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 34258.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_assignment_engine(&flags::new_registry());
        let item = new_forecast_assignment("code-470", "label-867");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
