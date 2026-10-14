use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the trust domain adjustment calculation.
pub trait TrustDomainAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainAdjustmentModifier for TieredTrustDomainAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAdjustmentGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainAdjustmentModifier for TrustDomainAdjustmentGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainAdjustment, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainAdjustmentAdjuster {
    pub minimum: f64,
}

impl TrustDomainAdjustmentModifier for MinimumTrustDomainAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainAdjustmentAdjuster {
    pub cap: f64,
}

impl TrustDomainAdjustmentModifier for CapTrustDomainAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the trust domain adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainAdjustmentEngine {
    pub steps: Vec<Rc<dyn TrustDomainAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainAdjustmentEngine {
    pub fn price(&mut self, item: &TrustDomainAdjustment) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_trust_domain_adjustment_engine() -> TrustDomainAdjustmentEngine {
    TrustDomainAdjustmentEngine {
        steps: vec![
            Rc::new(TieredTrustDomainAdjustmentAdjuster {
                bounds: vec![4050.0, 6792.0],
                rates: vec![1.1, 1.09],
            }) as Rc<dyn TrustDomainAdjustmentModifier>,
            Rc::new(TrustDomainAdjustmentGraceDaysOffsetAdjuster { weight: 0.08 }),
            Rc::new(MinimumTrustDomainAdjustmentAdjuster { minimum: 20.3 }),
            Rc::new(CapTrustDomainAdjustmentAdjuster { cap: 38560.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 2912.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_adjustment_engine();
        let item = new_trust_domain_adjustment("code-491", "name-479");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
