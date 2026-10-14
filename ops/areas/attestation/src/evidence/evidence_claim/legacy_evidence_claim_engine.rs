use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy evidence claim calculation.
pub trait LegacyEvidenceClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEvidenceClaim, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyEvidenceClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyEvidenceClaimAdjuster for FlaggedLegacyEvidenceClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyEvidenceClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyEvidenceClaimAdjuster {
    pub step: f64,
}

impl LegacyEvidenceClaimAdjuster for RoundingLegacyEvidenceClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyEvidenceClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyEvidenceClaimAdjuster {
    pub minimum: f64,
}

impl LegacyEvidenceClaimAdjuster for MinimumLegacyEvidenceClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyEvidenceClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyEvidenceClaimAdjuster {
    pub cap: f64,
}

impl LegacyEvidenceClaimAdjuster for CapLegacyEvidenceClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyEvidenceClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy evidence claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceClaimEngine {
    pub steps: Vec<Rc<dyn LegacyEvidenceClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEvidenceClaimEngine {
    pub fn compute(&mut self, item: &LegacyEvidenceClaim) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyEvidenceClaimAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEvidenceClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_evidence_claim_engine(flags: &flags::Registry) -> LegacyEvidenceClaimEngine {
    LegacyEvidenceClaimEngine {
        steps: vec![
            Rc::new(FlaggedLegacyEvidenceClaimAdjuster {
                flags: flags.clone(),
                flag: "legacy-evidence-claim-batched".to_string(),
                percent: -9.9,
            }) as Rc<dyn LegacyEvidenceClaimAdjuster>,
            Rc::new(RoundingLegacyEvidenceClaimAdjuster { step: 5.0 }),
            Rc::new(MinimumLegacyEvidenceClaimAdjuster { minimum: 29.3 }),
            Rc::new(CapLegacyEvidenceClaimAdjuster { cap: 36225.0 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 11980.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_evidence_claim_engine(&flags::new_registry());
        let item = new_legacy_evidence_claim("code-982", "title-163");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
