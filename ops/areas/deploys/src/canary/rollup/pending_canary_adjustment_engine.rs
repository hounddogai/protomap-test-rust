use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending canary adjustment calculation.
pub trait PendingCanaryAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingCanaryAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingCanaryAdjustmentAdjuster {
    pub percent: f64,
}

impl PendingCanaryAdjustmentModifier for PercentPendingCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingCanaryAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingCanaryAdjustmentAdjuster {
    pub step: f64,
}

impl PendingCanaryAdjustmentModifier for RoundingPendingCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingCanaryAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingCanaryAdjustmentAdjuster {
    pub minimum: f64,
}

impl PendingCanaryAdjustmentModifier for MinimumPendingCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingCanaryAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingCanaryAdjustmentCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl PendingCanaryAdjustmentModifier for PendingCanaryAdjustmentCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &PendingCanaryAdjustment, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingCanaryAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingCanaryAdjustmentModifier for TieredPendingCanaryAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingCanaryAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending canary adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingCanaryAdjustmentEngine {
    pub steps: Vec<Rc<dyn PendingCanaryAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingCanaryAdjustmentEngine {
    pub fn price(&mut self, item: &PendingCanaryAdjustment) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingCanaryAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingCanaryAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_canary_adjustment_engine() -> PendingCanaryAdjustmentEngine {
    PendingCanaryAdjustmentEngine {
        steps: vec![
            Rc::new(PercentPendingCanaryAdjustmentAdjuster { percent: 7.4 }) as Rc<dyn PendingCanaryAdjustmentModifier>,
            Rc::new(RoundingPendingCanaryAdjustmentAdjuster { step: 5.0 }),
            Rc::new(MinimumPendingCanaryAdjustmentAdjuster { minimum: 27.3 }),
            Rc::new(PendingCanaryAdjustmentCreatedAtOffsetAdjuster { weight: 0.73 }),
            Rc::new(TieredPendingCanaryAdjustmentAdjuster {
                bounds: vec![3836.0, 8128.0],
                rates: vec![0.84, 0.82],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 99520.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_canary_adjustment_engine();
        let item = new_pending_canary_adjustment("code-649", "title-865");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
