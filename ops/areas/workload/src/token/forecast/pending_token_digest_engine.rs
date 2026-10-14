use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending token digest calculation.
pub trait PendingTokenDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTokenDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingTokenDigestAdjuster {
    pub cap: f64,
}

impl PendingTokenDigestModifier for CapPendingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingTokenDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingTokenDigestAdjuster {
    pub minimum: f64,
}

impl PendingTokenDigestModifier for MinimumPendingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingTokenDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingTokenDigestAdjuster {
    pub percent: f64,
}

impl PendingTokenDigestModifier for PercentPendingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingTokenDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingTokenDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingTokenDigestModifier for SourcePendingTokenDigestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingTokenDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingTokenDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingTokenDigestModifier for TieredPendingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingTokenDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending token digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTokenDigestEngine {
    pub steps: Vec<Rc<dyn PendingTokenDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTokenDigestEngine {
    pub fn compute(&mut self, item: &PendingTokenDigest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingTokenDigestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingTokenDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_token_digest_engine() -> PendingTokenDigestEngine {
    PendingTokenDigestEngine {
        steps: vec![
            Rc::new(CapPendingTokenDigestAdjuster { cap: 24453.0 }) as Rc<dyn PendingTokenDigestModifier>,
            Rc::new(MinimumPendingTokenDigestAdjuster { minimum: 35.7 }),
            Rc::new(PercentPendingTokenDigestAdjuster { percent: 3.8 }),
            Rc::new(SourcePendingTokenDigestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.06)]),
            }),
            Rc::new(TieredPendingTokenDigestAdjuster {
                bounds: vec![3857.0, 4559.0, 9169.0],
                rates: vec![0.87, 0.93, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 83634.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_token_digest_engine();
        let item = new_pending_token_digest("code-394", "name-467");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
