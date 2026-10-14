use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending nonce claim calculation.
pub trait PendingNonceClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingNonceClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingNonceClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingNonceClaimStep for TieredPendingNonceClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingNonceClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingNonceClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingNonceClaimStep for SourcePendingNonceClaimAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingNonceClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingNonceClaimAdjuster {
    pub minimum: f64,
}

impl PendingNonceClaimStep for MinimumPendingNonceClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingNonceClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending nonce claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingNonceClaimEngine {
    pub steps: Vec<Rc<dyn PendingNonceClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingNonceClaimEngine {
    pub fn compute(&mut self, item: &PendingNonceClaim) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingNonceClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingNonceClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_nonce_claim_engine() -> PendingNonceClaimEngine {
    PendingNonceClaimEngine {
        steps: vec![
            Rc::new(TieredPendingNonceClaimAdjuster {
                bounds: vec![730.0, 1405.0, 3767.0, 9442.0],
                rates: vec![1.01, 1.13, 0.94, 1.19],
            }) as Rc<dyn PendingNonceClaimStep>,
            Rc::new(SourcePendingNonceClaimAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.26),
                    ("south".to_string(), 1.03),
                    ("online".to_string(), 1.09),
                ]),
            }),
            Rc::new(MinimumPendingNonceClaimAdjuster { minimum: 48.4 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 7897.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_nonce_claim_engine();
        let item = new_pending_nonce_claim("code-930", "name-938");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
