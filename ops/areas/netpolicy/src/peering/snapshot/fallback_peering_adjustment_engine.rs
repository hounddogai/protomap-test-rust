use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback peering adjustment calculation.
pub trait FallbackPeeringAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPeeringAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackPeeringAdjustmentAdjuster {
    pub cap: f64,
}

impl FallbackPeeringAdjustmentStep for CapFallbackPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackPeeringAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPeeringAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPeeringAdjustmentStep for TieredFallbackPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPeeringAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackPeeringAdjustmentAdjuster {
    pub percent: f64,
}

impl FallbackPeeringAdjustmentStep for PercentFallbackPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackPeeringAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPeeringAdjustmentAdjuster {
    pub step: f64,
}

impl FallbackPeeringAdjustmentStep for RoundingFallbackPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPeeringAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPeeringAdjustmentAdjuster {
    pub minimum: f64,
}

impl FallbackPeeringAdjustmentStep for MinimumFallbackPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPeeringAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleFallbackPeeringAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackPeeringAdjustmentStep for LocaleFallbackPeeringAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &FallbackPeeringAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback peering adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPeeringAdjustmentEngine {
    pub steps: Vec<Rc<dyn FallbackPeeringAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPeeringAdjustmentEngine {
    pub fn price(&mut self, item: &FallbackPeeringAdjustment) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPeeringAdjustmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPeeringAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_peering_adjustment_engine() -> FallbackPeeringAdjustmentEngine {
    FallbackPeeringAdjustmentEngine {
        steps: vec![
            Rc::new(CapFallbackPeeringAdjustmentAdjuster { cap: 37224.0 }) as Rc<dyn FallbackPeeringAdjustmentStep>,
            Rc::new(TieredFallbackPeeringAdjustmentAdjuster {
                bounds: vec![1959.0, 9805.0],
                rates: vec![0.87, 0.94],
            }),
            Rc::new(PercentFallbackPeeringAdjustmentAdjuster { percent: -7.4 }),
            Rc::new(RoundingFallbackPeeringAdjustmentAdjuster { step: 5.0 }),
            Rc::new(MinimumFallbackPeeringAdjustmentAdjuster { minimum: 9.5 }),
            Rc::new(LocaleFallbackPeeringAdjustmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.91)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 58338.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_peering_adjustment_engine();
        let item = new_fallback_peering_adjustment("code-83", "title-29");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
