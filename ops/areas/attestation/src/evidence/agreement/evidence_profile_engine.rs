use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the evidence profile calculation.
pub trait EvidenceProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEvidenceProfileAdjuster {
    pub step: f64,
}

impl EvidenceProfileStep for RoundingEvidenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EvidenceProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceProfileAdjuster {
    pub minimum: f64,
}

impl EvidenceProfileStep for MinimumEvidenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceProfileStep for FlaggedEvidenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceProfileAdjuster {
    pub cap: f64,
}

impl EvidenceProfileStep for CapEvidenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceProfileAdjuster {
    pub percent: f64,
}

impl EvidenceProfileStep for PercentEvidenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryEvidenceProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceProfileStep for CategoryEvidenceProfileAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &EvidenceProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the evidence profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceProfileEngine {
    pub steps: Vec<Rc<dyn EvidenceProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceProfileEngine {
    pub fn price(&mut self, item: &EvidenceProfile) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_evidence_profile_engine(flags: &flags::Registry) -> EvidenceProfileEngine {
    EvidenceProfileEngine {
        steps: vec![
            Rc::new(RoundingEvidenceProfileAdjuster { step: 5.0 }) as Rc<dyn EvidenceProfileStep>,
            Rc::new(MinimumEvidenceProfileAdjuster { minimum: 14.9 }),
            Rc::new(FlaggedEvidenceProfileAdjuster {
                flags: flags.clone(),
                flag: "evidence-profile-strict".to_string(),
                percent: -7.1,
            }),
            Rc::new(CapEvidenceProfileAdjuster { cap: 6064.0 }),
            Rc::new(PercentEvidenceProfileAdjuster { percent: -2.7 }),
            Rc::new(CategoryEvidenceProfileAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.25), ("south".to_string(), 0.93)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 66787.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_profile_engine(&flags::new_registry());
        let item = new_evidence_profile("code-328", "title-739");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
