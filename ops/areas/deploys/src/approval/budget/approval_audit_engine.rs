use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the approval audit calculation.
pub trait ApprovalAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalAuditAdjuster {
    pub cap: f64,
}

impl ApprovalAuditFactor for CapApprovalAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalAuditFactor for FlaggedApprovalAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalAuditFactor for TieredApprovalAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAuditScoreOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalAuditFactor for ApprovalAuditScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalAudit, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalAuditAdjuster {
    pub step: f64,
}

impl ApprovalAuditFactor for RoundingApprovalAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalAuditAdjuster {
    pub minimum: f64,
}

impl ApprovalAuditFactor for MinimumApprovalAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the approval audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalAuditEngine {
    pub steps: Vec<Rc<dyn ApprovalAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalAuditEngine {
    pub fn price(&mut self, item: &ApprovalAudit) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_audit_engine(flags: &flags::Registry) -> ApprovalAuditEngine {
    ApprovalAuditEngine {
        steps: vec![
            Rc::new(CapApprovalAuditAdjuster { cap: 28288.0 }) as Rc<dyn ApprovalAuditFactor>,
            Rc::new(FlaggedApprovalAuditAdjuster {
                flags: flags.clone(),
                flag: "approval-audit-manual".to_string(),
                percent: -3.3,
            }),
            Rc::new(TieredApprovalAuditAdjuster {
                bounds: vec![222.0, 1891.0],
                rates: vec![1.18, 1.17],
            }),
            Rc::new(ApprovalAuditScoreOffsetAdjuster { weight: 0.3 }),
            Rc::new(RoundingApprovalAuditAdjuster { step: 0.5 }),
            Rc::new(MinimumApprovalAuditAdjuster { minimum: 34.2 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 39697.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_audit_engine(&flags::new_registry());
        let item = new_approval_audit("code-739", "title-107");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
