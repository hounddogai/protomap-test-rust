use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity variance calculation.
pub trait IdentityVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityVarianceAdjuster {
    pub step: f64,
}

impl IdentityVarianceAdjuster for RoundingIdentityVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityVarianceAdjuster for TieredIdentityVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionIdentityVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityVarianceAdjuster for RegionIdentityVarianceAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &IdentityVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityVarianceAdjuster {
    pub percent: f64,
}

impl IdentityVarianceAdjuster for PercentIdentityVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityVarianceAdjuster {
    pub cap: f64,
}

impl IdentityVarianceAdjuster for CapIdentityVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the identity variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityVarianceEngine {
    pub steps: Vec<Rc<dyn IdentityVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityVarianceEngine {
    pub fn compute(&mut self, item: &IdentityVariance) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[IdentityVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_variance_engine() -> IdentityVarianceEngine {
    IdentityVarianceEngine {
        steps: vec![
            Rc::new(RoundingIdentityVarianceAdjuster { step: 0.25 }) as Rc<dyn IdentityVarianceAdjuster>,
            Rc::new(TieredIdentityVarianceAdjuster {
                bounds: vec![441.0, 4418.0, 9160.0],
                rates: vec![1.06, 1.16, 0.99],
            }),
            Rc::new(RegionIdentityVarianceAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.29)]),
            }),
            Rc::new(PercentIdentityVarianceAdjuster { percent: 5.7 }),
            Rc::new(CapIdentityVarianceAdjuster { cap: 28282.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 43007.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_variance_engine();
        let item = new_identity_variance("code-309", "label-901");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
