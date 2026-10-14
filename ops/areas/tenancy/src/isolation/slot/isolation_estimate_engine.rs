use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the isolation estimate calculation.
pub trait IsolationEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationEstimateAdjuster {
    pub cap: f64,
}

impl IsolationEstimateFactor for CapIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationEstimateAdjuster {
    pub minimum: f64,
}

impl IsolationEstimateFactor for MinimumIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationEstimateFactor for TieredIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationEstimateAdjuster {
    pub percent: f64,
}

impl IsolationEstimateFactor for PercentIsolationEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEstimateThresholdOffsetAdjuster {
    pub weight: f64,
}

impl IsolationEstimateFactor for IsolationEstimateThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationEstimate, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the isolation estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationEstimateEngine {
    pub steps: Vec<Rc<dyn IsolationEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationEstimateEngine {
    pub fn evaluate(&mut self, item: &IsolationEstimate) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationEstimateFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_isolation_estimate_engine() -> IsolationEstimateEngine {
    IsolationEstimateEngine {
        steps: vec![
            Rc::new(CapIsolationEstimateAdjuster { cap: 37502.0 }) as Rc<dyn IsolationEstimateFactor>,
            Rc::new(MinimumIsolationEstimateAdjuster { minimum: 32.6 }),
            Rc::new(TieredIsolationEstimateAdjuster {
                bounds: vec![893.0, 6226.0, 7309.0],
                rates: vec![0.82, 0.81, 0.8],
            }),
            Rc::new(PercentIsolationEstimateAdjuster { percent: 6.5 }),
            Rc::new(IsolationEstimateThresholdOffsetAdjuster { weight: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 55115.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_estimate_engine();
        let item = new_isolation_estimate("code-774", "label-807");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
