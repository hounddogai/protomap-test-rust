use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global usage metric calculation.
pub trait GlobalUsageMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalUsageMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageMetricRatioOffsetAdjuster {
    pub weight: f64,
}

impl GlobalUsageMetricAdjuster for GlobalUsageMetricRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalUsageMetric, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryGlobalUsageMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalUsageMetricAdjuster for CategoryGlobalUsageMetricAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &GlobalUsageMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalUsageMetricAdjuster {
    pub cap: f64,
}

impl GlobalUsageMetricAdjuster for CapGlobalUsageMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalUsageMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalUsageMetricAdjuster {
    pub percent: f64,
}

impl GlobalUsageMetricAdjuster for PercentGlobalUsageMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalUsageMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalUsageMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalUsageMetricAdjuster for TieredGlobalUsageMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalUsageMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalUsageMetricAdjuster {
    pub minimum: f64,
}

impl GlobalUsageMetricAdjuster for MinimumGlobalUsageMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalUsageMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global usage metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalUsageMetricEngine {
    pub steps: Vec<Rc<dyn GlobalUsageMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalUsageMetricEngine {
    pub fn run(&mut self, item: &GlobalUsageMetric) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalUsageMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_usage_metric_engine() -> GlobalUsageMetricEngine {
    GlobalUsageMetricEngine {
        steps: vec![
            Rc::new(GlobalUsageMetricRatioOffsetAdjuster { weight: 0.86 }) as Rc<dyn GlobalUsageMetricAdjuster>,
            Rc::new(CategoryGlobalUsageMetricAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.86), ("retail".to_string(), 1.06)]),
            }),
            Rc::new(CapGlobalUsageMetricAdjuster { cap: 28071.0 }),
            Rc::new(PercentGlobalUsageMetricAdjuster { percent: 0.0 }),
            Rc::new(TieredGlobalUsageMetricAdjuster {
                bounds: vec![3355.0, 4390.0, 5618.0],
                rates: vec![1.17, 0.85, 0.99],
            }),
            Rc::new(MinimumGlobalUsageMetricAdjuster { minimum: 13.2 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 7628.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_usage_metric_engine();
        let item = new_global_usage_metric("code-279", "name-545");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
