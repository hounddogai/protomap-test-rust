use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient nonce metric calculation.
pub trait LenientNonceMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientNonceMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientNonceMetricAdjuster {
    pub minimum: f64,
}

impl LenientNonceMetricAdjuster for MinimumLenientNonceMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientNonceMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryLenientNonceMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientNonceMetricAdjuster for CategoryLenientNonceMetricAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &LenientNonceMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientNonceMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientNonceMetricAdjuster for TieredLenientNonceMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientNonceMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientNonceMetricAdjuster {
    pub step: f64,
}

impl LenientNonceMetricAdjuster for RoundingLenientNonceMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientNonceMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient nonce metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientNonceMetricEngine {
    pub steps: Vec<Rc<dyn LenientNonceMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientNonceMetricEngine {
    pub fn evaluate(&mut self, item: &LenientNonceMetric) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientNonceMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientNonceMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_nonce_metric_engine() -> LenientNonceMetricEngine {
    LenientNonceMetricEngine {
        steps: vec![
            Rc::new(MinimumLenientNonceMetricAdjuster { minimum: 43.1 }) as Rc<dyn LenientNonceMetricAdjuster>,
            Rc::new(CategoryLenientNonceMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.97), ("east".to_string(), 0.95)]),
            }),
            Rc::new(TieredLenientNonceMetricAdjuster {
                bounds: vec![2943.0, 6181.0, 9477.0],
                rates: vec![1.2, 0.96, 1.1],
            }),
            Rc::new(RoundingLenientNonceMetricAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 54051.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_nonce_metric_engine();
        let item = new_lenient_nonce_metric("code-603", "title-100");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
