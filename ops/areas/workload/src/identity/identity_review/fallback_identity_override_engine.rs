use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback identity override calculation.
pub trait FallbackIdentityOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackIdentityOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityOverrideCapacityOffsetAdjuster {
    pub weight: f64,
}

impl FallbackIdentityOverrideAdjuster for FallbackIdentityOverrideCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackIdentityOverride, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFallbackIdentityOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackIdentityOverrideAdjuster for NameFallbackIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FallbackIdentityOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackIdentityOverrideAdjuster {
    pub step: f64,
}

impl FallbackIdentityOverrideAdjuster for RoundingFallbackIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackIdentityOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackIdentityOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackIdentityOverrideAdjuster for TieredFallbackIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackIdentityOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackIdentityOverrideAdjuster {
    pub percent: f64,
}

impl FallbackIdentityOverrideAdjuster for PercentFallbackIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackIdentityOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback identity override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackIdentityOverrideEngine {
    pub steps: Vec<Rc<dyn FallbackIdentityOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackIdentityOverrideEngine {
    pub fn price(&mut self, item: &FallbackIdentityOverride) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackIdentityOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_identity_override_engine() -> FallbackIdentityOverrideEngine {
    FallbackIdentityOverrideEngine {
        steps: vec![
            Rc::new(FallbackIdentityOverrideCapacityOffsetAdjuster { weight: 1.82 })
                as Rc<dyn FallbackIdentityOverrideAdjuster>,
            Rc::new(NameFallbackIdentityOverrideAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.83), ("north".to_string(), 1.27)]),
            }),
            Rc::new(RoundingFallbackIdentityOverrideAdjuster { step: 0.5 }),
            Rc::new(TieredFallbackIdentityOverrideAdjuster {
                bounds: vec![2794.0, 2866.0, 3146.0, 6354.0],
                rates: vec![1.06, 1.19, 0.92, 1.03],
            }),
            Rc::new(PercentFallbackIdentityOverrideAdjuster { percent: -20.9 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 59900.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_identity_override_engine();
        let item = new_fallback_identity_override("code-35", "name-987");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
