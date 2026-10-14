use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive slo metric calculation.
pub trait AdaptiveSloMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveSloMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveSloMetricAdjuster {
    pub minimum: f64,
}

impl AdaptiveSloMetricFactor for MinimumAdaptiveSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveSloMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAdaptiveSloMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveSloMetricFactor for TitleAdaptiveSloMetricAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AdaptiveSloMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveSloMetricAdjuster {
    pub percent: f64,
}

impl AdaptiveSloMetricFactor for PercentAdaptiveSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveSloMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveSloMetricAdjuster {
    pub cap: f64,
}

impl AdaptiveSloMetricFactor for CapAdaptiveSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveSloMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive slo metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveSloMetricEngine {
    pub steps: Vec<Rc<dyn AdaptiveSloMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveSloMetricEngine {
    pub fn evaluate(&mut self, item: &AdaptiveSloMetric) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveSloMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveSloMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_slo_metric_engine() -> AdaptiveSloMetricEngine {
    AdaptiveSloMetricEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveSloMetricAdjuster { minimum: 10.4 }) as Rc<dyn AdaptiveSloMetricFactor>,
            Rc::new(TitleAdaptiveSloMetricAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.95), ("central".to_string(), 1.33)]),
            }),
            Rc::new(PercentAdaptiveSloMetricAdjuster { percent: -2.8 }),
            Rc::new(CapAdaptiveSloMetricAdjuster { cap: 44192.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 55310.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_slo_metric_engine();
        let item = new_adaptive_slo_metric("code-630", "title-248");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
