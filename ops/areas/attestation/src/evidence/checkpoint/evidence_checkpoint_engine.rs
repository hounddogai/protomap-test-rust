use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the evidence checkpoint calculation.
pub trait EvidenceCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceCheckpointAdjuster {
    pub minimum: f64,
}

impl EvidenceCheckpointStep for MinimumEvidenceCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceCheckpointAdjuster {
    pub percent: f64,
}

impl EvidenceCheckpointStep for PercentEvidenceCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEvidenceCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceCheckpointStep for LabelEvidenceCheckpointAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EvidenceCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceCheckpointStep for FlaggedEvidenceCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceCheckpointStep for TieredEvidenceCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the evidence checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceCheckpointEngine {
    pub steps: Vec<Rc<dyn EvidenceCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceCheckpointEngine {
    pub fn price(&mut self, item: &EvidenceCheckpoint) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceCheckpointStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_evidence_checkpoint_engine(flags: &flags::Registry) -> EvidenceCheckpointEngine {
    EvidenceCheckpointEngine {
        steps: vec![
            Rc::new(MinimumEvidenceCheckpointAdjuster { minimum: 34.1 }) as Rc<dyn EvidenceCheckpointStep>,
            Rc::new(PercentEvidenceCheckpointAdjuster { percent: -21.6 }),
            Rc::new(LabelEvidenceCheckpointAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.71)]),
            }),
            Rc::new(FlaggedEvidenceCheckpointAdjuster {
                flags: flags.clone(),
                flag: "evidence-checkpoint-manual".to_string(),
                percent: -3.7,
            }),
            Rc::new(TieredEvidenceCheckpointAdjuster {
                bounds: vec![2527.0, 6029.0],
                rates: vec![0.82, 1.11],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 50026.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_checkpoint_engine(&flags::new_registry());
        let item = new_evidence_checkpoint("code-800", "label-29");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
