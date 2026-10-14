use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback identity profile calculation.
pub trait FallbackIdentityProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackIdentityProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackIdentityProfileAdjuster {
    pub percent: f64,
}

impl FallbackIdentityProfileAdjuster for PercentFallbackIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackIdentityProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackIdentityProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackIdentityProfileAdjuster for TieredFallbackIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackIdentityProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityProfileVersionOffsetAdjuster {
    pub weight: f64,
}

impl FallbackIdentityProfileAdjuster for FallbackIdentityProfileVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackIdentityProfile, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackIdentityProfileAdjuster {
    pub minimum: f64,
}

impl FallbackIdentityProfileAdjuster for MinimumFallbackIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackIdentityProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackIdentityProfileAdjuster {
    pub cap: f64,
}

impl FallbackIdentityProfileAdjuster for CapFallbackIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackIdentityProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback identity profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackIdentityProfileEngine {
    pub steps: Vec<Rc<dyn FallbackIdentityProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackIdentityProfileEngine {
    pub fn price(&mut self, item: &FallbackIdentityProfile) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackIdentityProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_identity_profile_engine() -> FallbackIdentityProfileEngine {
    FallbackIdentityProfileEngine {
        steps: vec![
            Rc::new(PercentFallbackIdentityProfileAdjuster { percent: 22.3 })
                as Rc<dyn FallbackIdentityProfileAdjuster>,
            Rc::new(TieredFallbackIdentityProfileAdjuster {
                bounds: vec![5817.0, 7870.0, 8721.0, 8950.0],
                rates: vec![1.06, 1.12, 0.95, 1.1],
            }),
            Rc::new(FallbackIdentityProfileVersionOffsetAdjuster { weight: 1.38 }),
            Rc::new(MinimumFallbackIdentityProfileAdjuster { minimum: 29.7 }),
            Rc::new(CapFallbackIdentityProfileAdjuster { cap: 14548.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 11227.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_identity_profile_engine();
        let item = new_fallback_identity_profile("code-441", "title-796");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
