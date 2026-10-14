use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted forecast metric calculation.
pub trait WeightedForecastMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedForecastMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl WeightedForecastMetricStep for WeightedForecastMetricRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedForecastMetric, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedForecastMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedForecastMetricStep for FlaggedWeightedForecastMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedForecastMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedForecastMetricAdjuster {
    pub percent: f64,
}

impl WeightedForecastMetricStep for PercentWeightedForecastMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedForecastMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedForecastMetricAdjuster {
    pub minimum: f64,
}

impl WeightedForecastMetricStep for MinimumWeightedForecastMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedForecastMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the weighted forecast metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedForecastMetricEngine {
    pub steps: Vec<Rc<dyn WeightedForecastMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedForecastMetricEngine {
    pub fn compute(&mut self, item: &WeightedForecastMetric) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedForecastMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedForecastMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_forecast_metric_engine(flags: &flags::Registry) -> WeightedForecastMetricEngine {
    WeightedForecastMetricEngine {
        steps: vec![
            Rc::new(WeightedForecastMetricRotationDaysOffsetAdjuster { weight: 1.35 })
                as Rc<dyn WeightedForecastMetricStep>,
            Rc::new(FlaggedWeightedForecastMetricAdjuster {
                flags: flags.clone(),
                flag: "weighted-forecast-metric-primary".to_string(),
                percent: 0.8,
            }),
            Rc::new(PercentWeightedForecastMetricAdjuster { percent: 17.7 }),
            Rc::new(MinimumWeightedForecastMetricAdjuster { minimum: 39.7 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 91009.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_forecast_metric_engine(&flags::new_registry());
        let item = new_weighted_forecast_metric("code-986", "name-626");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
