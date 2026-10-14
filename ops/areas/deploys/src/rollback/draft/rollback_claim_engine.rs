use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rollback claim calculation.
pub trait RollbackClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackClaimAdjuster {
    pub minimum: f64,
}

impl RollbackClaimModifier for MinimumRollbackClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollbackClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollbackClaimModifier for FlaggedRollbackClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollbackClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackClaimScoreOffsetAdjuster {
    pub weight: f64,
}

impl RollbackClaimModifier for RollbackClaimScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackClaim, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackClaimAdjuster {
    pub cap: f64,
}

impl RollbackClaimModifier for CapRollbackClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rollback claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackClaimEngine {
    pub steps: Vec<Rc<dyn RollbackClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackClaimEngine {
    pub fn price(&mut self, item: &RollbackClaim) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rollback_claim_engine(flags: &flags::Registry) -> RollbackClaimEngine {
    RollbackClaimEngine {
        steps: vec![
            Rc::new(MinimumRollbackClaimAdjuster { minimum: 43.7 }) as Rc<dyn RollbackClaimModifier>,
            Rc::new(FlaggedRollbackClaimAdjuster {
                flags: flags.clone(),
                flag: "rollback-claim-default".to_string(),
                percent: 7.6,
            }),
            Rc::new(RollbackClaimScoreOffsetAdjuster { weight: 0.95 }),
            Rc::new(CapRollbackClaimAdjuster { cap: 9621.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 8660.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_claim_engine(&flags::new_registry());
        let item = new_rollback_claim("code-727", "title-235");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
