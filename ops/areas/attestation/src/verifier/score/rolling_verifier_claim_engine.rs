use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling verifier claim calculation.
pub trait RollingVerifierClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingVerifierClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierClaimMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RollingVerifierClaimFactor for RollingVerifierClaimMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RollingVerifierClaim, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingVerifierClaimAdjuster {
    pub minimum: f64,
}

impl RollingVerifierClaimFactor for MinimumRollingVerifierClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingVerifierClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRollingVerifierClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingVerifierClaimFactor for UnitRollingVerifierClaimAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RollingVerifierClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rolling verifier claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingVerifierClaimEngine {
    pub steps: Vec<Rc<dyn RollingVerifierClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingVerifierClaimEngine {
    pub fn compute(&mut self, item: &RollingVerifierClaim) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingVerifierClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingVerifierClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_verifier_claim_engine() -> RollingVerifierClaimEngine {
    RollingVerifierClaimEngine {
        steps: vec![
            Rc::new(RollingVerifierClaimMaximumOffsetAdjuster { weight: 1.02 }) as Rc<dyn RollingVerifierClaimFactor>,
            Rc::new(MinimumRollingVerifierClaimAdjuster { minimum: 47.9 }),
            Rc::new(UnitRollingVerifierClaimAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.01),
                    ("south".to_string(), 0.91),
                    ("east".to_string(), 1.01),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 69850.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_verifier_claim_engine();
        let item = new_rolling_verifier_claim("code-208", "name-856");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
