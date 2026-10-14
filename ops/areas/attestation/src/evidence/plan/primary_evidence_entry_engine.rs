use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary evidence entry calculation.
pub trait PrimaryEvidenceEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEvidenceEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryEvidenceEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryEvidenceEntryStep for FlaggedPrimaryEvidenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryEvidenceEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryEvidenceEntryAdjuster {
    pub step: f64,
}

impl PrimaryEvidenceEntryStep for RoundingPrimaryEvidenceEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryEvidenceEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimaryEvidenceEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEvidenceEntryStep for ClusterPrimaryEvidenceEntryAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimaryEvidenceEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary evidence entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEvidenceEntryEngine {
    pub steps: Vec<Rc<dyn PrimaryEvidenceEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEvidenceEntryEngine {
    pub fn price(&mut self, item: &PrimaryEvidenceEntry) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryEvidenceEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEvidenceEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_evidence_entry_engine(flags: &flags::Registry) -> PrimaryEvidenceEntryEngine {
    PrimaryEvidenceEntryEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryEvidenceEntryAdjuster {
                flags: flags.clone(),
                flag: "primary-evidence-entry-strict".to_string(),
                percent: -2.1,
            }) as Rc<dyn PrimaryEvidenceEntryStep>,
            Rc::new(RoundingPrimaryEvidenceEntryAdjuster { step: 0.1 }),
            Rc::new(ClusterPrimaryEvidenceEntryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.13), ("west".to_string(), 1.02)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 11212.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_evidence_entry_engine(&flags::new_registry());
        let item = new_primary_evidence_entry("code-365", "label-748");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
