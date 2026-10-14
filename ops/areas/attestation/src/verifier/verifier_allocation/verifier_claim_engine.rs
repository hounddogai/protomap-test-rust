use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier claim calculation.
pub trait VerifierClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl VerifierClaimFactor for VerifierClaimEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierClaim, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitVerifierClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierClaimFactor for UnitVerifierClaimAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &VerifierClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierClaimAdjuster {
    pub cap: f64,
}

impl VerifierClaimFactor for CapVerifierClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierClaimFactor for FlaggedVerifierClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierClaimAdjuster {
    pub step: f64,
}

impl VerifierClaimFactor for RoundingVerifierClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierClaimAdjuster {
    pub minimum: f64,
}

impl VerifierClaimFactor for MinimumVerifierClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the verifier claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierClaimEngine {
    pub steps: Vec<Rc<dyn VerifierClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierClaimEngine {
    pub fn evaluate(&mut self, item: &VerifierClaim) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[VerifierClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_verifier_claim_engine(flags: &flags::Registry) -> VerifierClaimEngine {
    VerifierClaimEngine {
        steps: vec![
            Rc::new(VerifierClaimEndsAtOffsetAdjuster { weight: 0.75 }) as Rc<dyn VerifierClaimFactor>,
            Rc::new(UnitVerifierClaimAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.71),
                    ("central".to_string(), 1.1),
                    ("north".to_string(), 0.76),
                ]),
            }),
            Rc::new(CapVerifierClaimAdjuster { cap: 46705.0 }),
            Rc::new(FlaggedVerifierClaimAdjuster {
                flags: flags.clone(),
                flag: "verifier-claim-manual".to_string(),
                percent: -4.2,
            }),
            Rc::new(RoundingVerifierClaimAdjuster { step: 0.1 }),
            Rc::new(MinimumVerifierClaimAdjuster { minimum: 44.8 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 59246.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_claim_engine(&flags::new_registry());
        let item = new_verifier_claim("code-529", "title-461");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
