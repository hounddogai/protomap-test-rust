use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the nonce claim calculation.
pub trait NonceClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceClaimModifier for FlaggedNonceClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceClaimAdjuster {
    pub cap: f64,
}

impl NonceClaimModifier for CapNonceClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumNonceClaimAdjuster {
    pub minimum: f64,
}

impl NonceClaimModifier for MinimumNonceClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &NonceClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceClaimEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl NonceClaimModifier for NonceClaimEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &NonceClaim, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceClaimModifier for TieredNonceClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the nonce claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceClaimEngine {
    pub steps: Vec<Rc<dyn NonceClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceClaimEngine {
    pub fn evaluate(&mut self, item: &NonceClaim) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_claim_engine(flags: &flags::Registry) -> NonceClaimEngine {
    NonceClaimEngine {
        steps: vec![
            Rc::new(FlaggedNonceClaimAdjuster {
                flags: flags.clone(),
                flag: "nonce-claim-regional".to_string(),
                percent: 6.8,
            }) as Rc<dyn NonceClaimModifier>,
            Rc::new(CapNonceClaimAdjuster { cap: 6143.0 }),
            Rc::new(MinimumNonceClaimAdjuster { minimum: 41.1 }),
            Rc::new(NonceClaimEndsAtOffsetAdjuster { weight: 0.26 }),
            Rc::new(TieredNonceClaimAdjuster {
                bounds: vec![1609.0, 2653.0, 8944.0],
                rates: vec![1.01, 0.86, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 14195.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_claim_engine(&flags::new_registry());
        let item = new_nonce_claim("code-856", "label-470");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
