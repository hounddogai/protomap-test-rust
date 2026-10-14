use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback usage variance calculation.
pub trait FallbackUsageVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackUsageVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageVarianceExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl FallbackUsageVarianceModifier for FallbackUsageVarianceExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackUsageVariance, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackUsageVarianceAdjuster {
    pub step: f64,
}

impl FallbackUsageVarianceModifier for RoundingFallbackUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackUsageVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFallbackUsageVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackUsageVarianceModifier for ClusterFallbackUsageVarianceAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FallbackUsageVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackUsageVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackUsageVarianceModifier for TieredFallbackUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackUsageVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackUsageVarianceAdjuster {
    pub minimum: f64,
}

impl FallbackUsageVarianceModifier for MinimumFallbackUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackUsageVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackUsageVarianceAdjuster {
    pub percent: f64,
}

impl FallbackUsageVarianceModifier for PercentFallbackUsageVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackUsageVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback usage variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackUsageVarianceEngine {
    pub steps: Vec<Rc<dyn FallbackUsageVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackUsageVarianceEngine {
    pub fn price(&mut self, item: &FallbackUsageVariance) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[FallbackUsageVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_usage_variance_engine() -> FallbackUsageVarianceEngine {
    FallbackUsageVarianceEngine {
        steps: vec![
            Rc::new(FallbackUsageVarianceExpiresAtOffsetAdjuster { weight: 1.38 })
                as Rc<dyn FallbackUsageVarianceModifier>,
            Rc::new(RoundingFallbackUsageVarianceAdjuster { step: 0.5 }),
            Rc::new(ClusterFallbackUsageVarianceAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.1), ("north".to_string(), 0.76)]),
            }),
            Rc::new(TieredFallbackUsageVarianceAdjuster {
                bounds: vec![243.0, 3919.0, 5821.0, 7252.0],
                rates: vec![1.08, 0.92, 1.19, 0.88],
            }),
            Rc::new(MinimumFallbackUsageVarianceAdjuster { minimum: 13.0 }),
            Rc::new(PercentFallbackUsageVarianceAdjuster { percent: 3.8 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 16157.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_usage_variance_engine();
        let item = new_fallback_usage_variance("code-759", "label-816");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
