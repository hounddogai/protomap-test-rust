use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier draft calculation.
pub trait VerifierDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierDraftModifier for TieredVerifierDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierDraftModifier for FlaggedVerifierDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierDraftAdjuster {
    pub minimum: f64,
}

impl VerifierDraftModifier for MinimumVerifierDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierDraftAdjuster {
    pub step: f64,
}

impl VerifierDraftModifier for RoundingVerifierDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the verifier draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierDraftEngine {
    pub steps: Vec<Rc<dyn VerifierDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierDraftEngine {
    pub fn compute(&mut self, item: &VerifierDraft) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_verifier_draft_engine(flags: &flags::Registry) -> VerifierDraftEngine {
    VerifierDraftEngine {
        steps: vec![
            Rc::new(TieredVerifierDraftAdjuster {
                bounds: vec![4341.0, 5759.0, 7745.0],
                rates: vec![1.01, 1.11, 1.08],
            }) as Rc<dyn VerifierDraftModifier>,
            Rc::new(FlaggedVerifierDraftAdjuster {
                flags: flags.clone(),
                flag: "verifier-draft-tiered".to_string(),
                percent: -7.0,
            }),
            Rc::new(MinimumVerifierDraftAdjuster { minimum: 26.6 }),
            Rc::new(RoundingVerifierDraftAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 25792.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_draft_engine(&flags::new_registry());
        let item = new_verifier_draft("code-239", "title-481");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
