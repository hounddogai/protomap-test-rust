use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary evidence notice calculation.
pub trait PrimaryEvidenceNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEvidenceNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceNoticeVersionOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryEvidenceNoticeStep for PrimaryEvidenceNoticeVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryEvidenceNotice, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryEvidenceNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryEvidenceNoticeStep for FlaggedPrimaryEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryEvidenceNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePrimaryEvidenceNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEvidenceNoticeStep for ScopePrimaryEvidenceNoticeAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PrimaryEvidenceNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary evidence notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEvidenceNoticeEngine {
    pub steps: Vec<Rc<dyn PrimaryEvidenceNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEvidenceNoticeEngine {
    pub fn run(&mut self, item: &PrimaryEvidenceNotice) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryEvidenceNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEvidenceNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_evidence_notice_engine(flags: &flags::Registry) -> PrimaryEvidenceNoticeEngine {
    PrimaryEvidenceNoticeEngine {
        steps: vec![
            Rc::new(PrimaryEvidenceNoticeVersionOffsetAdjuster { weight: 1.28 }) as Rc<dyn PrimaryEvidenceNoticeStep>,
            Rc::new(FlaggedPrimaryEvidenceNoticeAdjuster {
                flags: flags.clone(),
                flag: "primary-evidence-notice-global".to_string(),
                percent: 5.9,
            }),
            Rc::new(ScopePrimaryEvidenceNoticeAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.33),
                    ("retail".to_string(), 0.96),
                    ("central".to_string(), 0.85),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 46042.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_evidence_notice_engine(&flags::new_registry());
        let item = new_primary_evidence_notice("code-788", "name-757");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
