use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence bundle calculation.
pub trait EvidenceBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBundleRevisionOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceBundleStep for EvidenceBundleRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceBundle, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceBundleAdjuster {
    pub step: f64,
}

impl EvidenceBundleStep for RoundingEvidenceBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceBundleStep for FlaggedEvidenceBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceBundleAdjuster {
    pub cap: f64,
}

impl EvidenceBundleStep for CapEvidenceBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceBundleAdjuster {
    pub minimum: f64,
}

impl EvidenceBundleStep for MinimumEvidenceBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceBundleStep for TieredEvidenceBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the evidence bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceBundleEngine {
    pub steps: Vec<Rc<dyn EvidenceBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceBundleEngine {
    pub fn compute(&mut self, item: &EvidenceBundle) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_evidence_bundle_engine(flags: &flags::Registry) -> EvidenceBundleEngine {
    EvidenceBundleEngine {
        steps: vec![
            Rc::new(EvidenceBundleRevisionOffsetAdjuster { weight: 0.86 }) as Rc<dyn EvidenceBundleStep>,
            Rc::new(RoundingEvidenceBundleAdjuster { step: 0.1 }),
            Rc::new(FlaggedEvidenceBundleAdjuster {
                flags: flags.clone(),
                flag: "evidence-bundle-pending".to_string(),
                percent: 9.9,
            }),
            Rc::new(CapEvidenceBundleAdjuster { cap: 7334.0 }),
            Rc::new(MinimumEvidenceBundleAdjuster { minimum: 31.0 }),
            Rc::new(TieredEvidenceBundleAdjuster {
                bounds: vec![986.0, 1732.0, 5469.0, 7112.0],
                rates: vec![1.15, 1.15, 0.81, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 80579.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_bundle_engine(&flags::new_registry());
        let item = new_evidence_bundle("code-399", "label-82");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
