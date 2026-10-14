use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the nonce adjustment calculation.
pub trait NonceAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceAdjustmentAdjuster {
    pub percent: f64,
}

impl NonceAdjustmentStep for PercentNonceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceAdjustmentAdjuster {
    pub minimum: f64,
}

impl NonceAdjustmentStep for MinimumNonceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceAdjustmentAdjuster {
    pub cap: f64,
}

impl NonceAdjustmentStep for CapNonceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceAdjustmentStep for TieredNonceAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the nonce adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceAdjustmentEngine {
    pub steps: Vec<Rc<dyn NonceAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceAdjustmentEngine {
    pub fn compute(&mut self, item: &NonceAdjustment) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceAdjustmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_adjustment_engine() -> NonceAdjustmentEngine {
    NonceAdjustmentEngine {
        steps: vec![
            Rc::new(PercentNonceAdjustmentAdjuster { percent: -3.9 }) as Rc<dyn NonceAdjustmentStep>,
            Rc::new(MinimumNonceAdjustmentAdjuster { minimum: 2.9 }),
            Rc::new(CapNonceAdjustmentAdjuster { cap: 3202.0 }),
            Rc::new(TieredNonceAdjustmentAdjuster {
                bounds: vec![1089.0, 9033.0, 9627.0],
                rates: vec![0.92, 0.93, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 64339.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_adjustment_engine();
        let item = new_nonce_adjustment("code-410", "name-692");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
