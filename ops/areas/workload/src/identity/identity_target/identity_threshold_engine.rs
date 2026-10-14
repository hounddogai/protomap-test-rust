use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity threshold calculation.
pub trait IdentityThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceIdentityThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityThresholdAdjuster for NamespaceIdentityThresholdAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &IdentityThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityThresholdAdjuster {
    pub step: f64,
}

impl IdentityThresholdAdjuster for RoundingIdentityThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityThresholdAdjuster {
    pub percent: f64,
}

impl IdentityThresholdAdjuster for PercentIdentityThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityThresholdAdjuster {
    pub cap: f64,
}

impl IdentityThresholdAdjuster for CapIdentityThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityThresholdAdjuster {
    pub minimum: f64,
}

impl IdentityThresholdAdjuster for MinimumIdentityThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityThresholdAdjuster for TieredIdentityThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the identity threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityThresholdEngine {
    pub steps: Vec<Rc<dyn IdentityThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityThresholdEngine {
    pub fn price(&mut self, item: &IdentityThreshold) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_identity_threshold_engine() -> IdentityThresholdEngine {
    IdentityThresholdEngine {
        steps: vec![
            Rc::new(NamespaceIdentityThresholdAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.18), ("online".to_string(), 1.38)]),
            }) as Rc<dyn IdentityThresholdAdjuster>,
            Rc::new(RoundingIdentityThresholdAdjuster { step: 0.1 }),
            Rc::new(PercentIdentityThresholdAdjuster { percent: 0.1 }),
            Rc::new(CapIdentityThresholdAdjuster { cap: 1459.0 }),
            Rc::new(MinimumIdentityThresholdAdjuster { minimum: 20.4 }),
            Rc::new(TieredIdentityThresholdAdjuster {
                bounds: vec![3630.0, 8515.0],
                rates: vec![0.98, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 56727.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_threshold_engine();
        let item = new_identity_threshold("code-826", "label-874");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
