use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast allocation calculation.
pub trait ForecastAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionForecastAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastAllocationAdjuster for RegionForecastAllocationAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ForecastAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastAllocationAdjuster {
    pub minimum: f64,
}

impl ForecastAllocationAdjuster for MinimumForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastAllocationAdjuster {
    pub cap: f64,
}

impl ForecastAllocationAdjuster for CapForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastAllocationAdjuster for TieredForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentForecastAllocationAdjuster {
    pub percent: f64,
}

impl ForecastAllocationAdjuster for PercentForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ForecastAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastAllocationAdjuster {
    pub step: f64,
}

impl ForecastAllocationAdjuster for RoundingForecastAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the forecast allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastAllocationEngine {
    pub steps: Vec<Rc<dyn ForecastAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastAllocationEngine {
    pub fn price(&mut self, item: &ForecastAllocation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ForecastAllocationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ForecastAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_forecast_allocation_engine() -> ForecastAllocationEngine {
    ForecastAllocationEngine {
        steps: vec![
            Rc::new(RegionForecastAllocationAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.01),
                    ("south".to_string(), 1.14),
                    ("north".to_string(), 1.01),
                ]),
            }) as Rc<dyn ForecastAllocationAdjuster>,
            Rc::new(MinimumForecastAllocationAdjuster { minimum: 26.0 }),
            Rc::new(CapForecastAllocationAdjuster { cap: 5865.0 }),
            Rc::new(TieredForecastAllocationAdjuster {
                bounds: vec![522.0, 538.0, 7212.0, 8892.0],
                rates: vec![1.15, 0.84, 1.17, 0.88],
            }),
            Rc::new(PercentForecastAllocationAdjuster { percent: 23.6 }),
            Rc::new(RoundingForecastAllocationAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 97387.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_allocation_engine();
        let item = new_forecast_allocation("code-978", "name-787");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
