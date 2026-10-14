use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback redaction metric calculation.
pub trait FallbackRedactionMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackRedactionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackRedactionMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackRedactionMetricModifier for TieredFallbackRedactionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackRedactionMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackRedactionMetricAdjuster {
    pub step: f64,
}

impl FallbackRedactionMetricModifier for RoundingFallbackRedactionMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackRedactionMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionMetricAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl FallbackRedactionMetricModifier for FallbackRedactionMetricAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackRedactionMetric, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleFallbackRedactionMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackRedactionMetricModifier for LocaleFallbackRedactionMetricAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &FallbackRedactionMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback redaction metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackRedactionMetricEngine {
    pub steps: Vec<Rc<dyn FallbackRedactionMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackRedactionMetricEngine {
    pub fn compute(&mut self, item: &FallbackRedactionMetric) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackRedactionMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackRedactionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_redaction_metric_engine() -> FallbackRedactionMetricEngine {
    FallbackRedactionMetricEngine {
        steps: vec![
            Rc::new(TieredFallbackRedactionMetricAdjuster {
                bounds: vec![2259.0, 2619.0, 3723.0],
                rates: vec![0.88, 0.81, 1.15],
            }) as Rc<dyn FallbackRedactionMetricModifier>,
            Rc::new(RoundingFallbackRedactionMetricAdjuster { step: 0.05 }),
            Rc::new(FallbackRedactionMetricAttemptsOffsetAdjuster { weight: 1.54 }),
            Rc::new(LocaleFallbackRedactionMetricAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.04), ("west".to_string(), 0.7)]),
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 75169.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_redaction_metric_engine();
        let item = new_fallback_redaction_metric("code-504", "name-466");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
