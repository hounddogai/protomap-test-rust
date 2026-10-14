use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the evidence audit calculation.
pub trait EvidenceAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EvidenceAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEvidenceAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EvidenceAuditFactor for TieredEvidenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EvidenceAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEvidenceAuditAdjuster {
    pub percent: f64,
}

impl EvidenceAuditFactor for PercentEvidenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EvidenceAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEvidenceAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EvidenceAuditFactor for FlaggedEvidenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EvidenceAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameEvidenceAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EvidenceAuditFactor for NameEvidenceAuditAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &EvidenceAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEvidenceAuditAdjuster {
    pub minimum: f64,
}

impl EvidenceAuditFactor for MinimumEvidenceAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EvidenceAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the evidence audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EvidenceAuditEngine {
    pub steps: Vec<Rc<dyn EvidenceAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EvidenceAuditEngine {
    pub fn compute(&mut self, item: &EvidenceAudit) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EvidenceAuditFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EvidenceAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_evidence_audit_engine(flags: &flags::Registry) -> EvidenceAuditEngine {
    EvidenceAuditEngine {
        steps: vec![
            Rc::new(TieredEvidenceAuditAdjuster {
                bounds: vec![166.0, 7829.0],
                rates: vec![0.82, 1.0],
            }) as Rc<dyn EvidenceAuditFactor>,
            Rc::new(PercentEvidenceAuditAdjuster { percent: -15.4 }),
            Rc::new(FlaggedEvidenceAuditAdjuster {
                flags: flags.clone(),
                flag: "evidence-audit-weighted".to_string(),
                percent: -7.3,
            }),
            Rc::new(NameEvidenceAuditAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.2),
                    ("online".to_string(), 1.27),
                    ("west".to_string(), 1.27),
                ]),
            }),
            Rc::new(MinimumEvidenceAuditAdjuster { minimum: 31.3 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 98456.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_evidence_audit_engine(&flags::new_registry());
        let item = new_evidence_audit("code-881", "name-719");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
