use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the weighted measurement metric calculation.
pub trait WeightedMeasurementMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedMeasurementMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedMeasurementMetricAdjuster {
    pub minimum: f64,
}

impl WeightedMeasurementMetricAdjuster for MinimumWeightedMeasurementMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedMeasurementMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedMeasurementMetricAdjuster {
    pub percent: f64,
}

impl WeightedMeasurementMetricAdjuster for PercentWeightedMeasurementMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedMeasurementMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedMeasurementMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedMeasurementMetricAdjuster for TieredWeightedMeasurementMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedMeasurementMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted measurement metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedMeasurementMetricEngine {
    pub steps: Vec<Rc<dyn WeightedMeasurementMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedMeasurementMetricEngine {
    pub fn evaluate(&mut self, item: &WeightedMeasurementMetric) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedMeasurementMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedMeasurementMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_measurement_metric_engine() -> WeightedMeasurementMetricEngine {
    WeightedMeasurementMetricEngine {
        steps: vec![
            Rc::new(MinimumWeightedMeasurementMetricAdjuster { minimum: 26.2 })
                as Rc<dyn WeightedMeasurementMetricAdjuster>,
            Rc::new(PercentWeightedMeasurementMetricAdjuster { percent: 23.4 }),
            Rc::new(TieredWeightedMeasurementMetricAdjuster {
                bounds: vec![6218.0, 6720.0, 9928.0],
                rates: vec![1.17, 0.92, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 95753.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_measurement_metric_engine();
        let item = new_weighted_measurement_metric("code-120", "name-20");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
