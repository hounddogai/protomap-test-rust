use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional freeze metric calculation.
pub trait RegionalFreezeMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalFreezeMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalFreezeMetricAdjuster {
    pub percent: f64,
}

impl RegionalFreezeMetricFactor for PercentRegionalFreezeMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalFreezeMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeMetricCostOffsetAdjuster {
    pub weight: f64,
}

impl RegionalFreezeMetricFactor for RegionalFreezeMetricCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalFreezeMetric, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalFreezeMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalFreezeMetricFactor for TieredRegionalFreezeMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalFreezeMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalFreezeMetricAdjuster {
    pub cap: f64,
}

impl RegionalFreezeMetricFactor for CapRegionalFreezeMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalFreezeMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the regional freeze metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalFreezeMetricEngine {
    pub steps: Vec<Rc<dyn RegionalFreezeMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalFreezeMetricEngine {
    pub fn evaluate(&mut self, item: &RegionalFreezeMetric) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalFreezeMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalFreezeMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_freeze_metric_engine() -> RegionalFreezeMetricEngine {
    RegionalFreezeMetricEngine {
        steps: vec![
            Rc::new(PercentRegionalFreezeMetricAdjuster { percent: -8.1 }) as Rc<dyn RegionalFreezeMetricFactor>,
            Rc::new(RegionalFreezeMetricCostOffsetAdjuster { weight: 0.08 }),
            Rc::new(TieredRegionalFreezeMetricAdjuster {
                bounds: vec![545.0, 7954.0, 8690.0],
                rates: vec![1.15, 1.08, 1.11],
            }),
            Rc::new(CapRegionalFreezeMetricAdjuster { cap: 3736.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 38356.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_freeze_metric_engine();
        let item = new_regional_freeze_metric("code-866", "title-203");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
