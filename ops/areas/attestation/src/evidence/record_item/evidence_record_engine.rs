use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the evidence record calculation.
pub trait EvidenceRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRecordGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl EvidenceRecordModifier for EvidenceRecordGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &EvidenceRecord, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceRecordModifier for FlaggedEvidenceRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEvidenceRecordAdjuster {
    pub cap: f64,
}

impl EvidenceRecordModifier for CapEvidenceRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EvidenceRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceRecordModifier for TieredEvidenceRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEvidenceRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceRecordModifier for LabelEvidenceRecordAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EvidenceRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the evidence record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceRecordEngine {
    pub steps: Vec<Rc<dyn EvidenceRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceRecordEngine {
    pub fn evaluate(&mut self, item: &EvidenceRecord) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_evidence_record_engine(flags: &flags::Registry) -> EvidenceRecordEngine {
    EvidenceRecordEngine {
        steps: vec![
            Rc::new(EvidenceRecordGraceDaysOffsetAdjuster { weight: 0.65 }) as Rc<dyn EvidenceRecordModifier>,
            Rc::new(FlaggedEvidenceRecordAdjuster {
                flags: flags.clone(),
                flag: "evidence-record-strict".to_string(),
                percent: -2.4,
            }),
            Rc::new(CapEvidenceRecordAdjuster { cap: 6240.0 }),
            Rc::new(TieredEvidenceRecordAdjuster {
                bounds: vec![2164.0, 8130.0],
                rates: vec![0.91, 0.89],
            }),
            Rc::new(LabelEvidenceRecordAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.26),
                    ("central".to_string(), 0.96),
                    ("online".to_string(), 1.31),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 14224.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_record_engine(&flags::new_registry());
        let item = new_evidence_record("code-240", "label-702");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
