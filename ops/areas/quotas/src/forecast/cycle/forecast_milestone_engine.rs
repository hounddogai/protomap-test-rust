use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the forecast milestone calculation.
pub trait ForecastMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastMilestoneModifier for FlaggedForecastMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastMilestoneModifier for TieredForecastMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastMilestoneAdjuster {
    pub minimum: f64,
}

impl ForecastMilestoneModifier for MinimumForecastMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMilestoneScoreOffsetAdjuster {
    pub weight: f64,
}

impl ForecastMilestoneModifier for ForecastMilestoneScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastMilestone, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the forecast milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastMilestoneEngine {
    pub steps: Vec<Rc<dyn ForecastMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastMilestoneEngine {
    pub fn evaluate(&mut self, item: &ForecastMilestone) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastMilestoneModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_forecast_milestone_engine(flags: &flags::Registry) -> ForecastMilestoneEngine {
    ForecastMilestoneEngine {
        steps: vec![
            Rc::new(FlaggedForecastMilestoneAdjuster {
                flags: flags.clone(),
                flag: "forecast-milestone-seasonal".to_string(),
                percent: -6.4,
            }) as Rc<dyn ForecastMilestoneModifier>,
            Rc::new(TieredForecastMilestoneAdjuster {
                bounds: vec![2688.0, 3640.0, 5755.0, 6958.0],
                rates: vec![1.09, 0.87, 0.85, 0.86],
            }),
            Rc::new(MinimumForecastMilestoneAdjuster { minimum: 24.9 }),
            Rc::new(ForecastMilestoneScoreOffsetAdjuster { weight: 0.45 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 58183.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_milestone_engine(&flags::new_registry());
        let item = new_forecast_milestone("code-274", "label-250");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
