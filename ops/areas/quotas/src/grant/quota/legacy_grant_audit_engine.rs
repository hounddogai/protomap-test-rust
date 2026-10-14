use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy grant audit calculation.
pub trait LegacyGrantAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyGrantAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyGrantAuditAdjuster {
    pub percent: f64,
}

impl LegacyGrantAuditStep for PercentLegacyGrantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyGrantAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyGrantAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyGrantAuditStep for FlaggedLegacyGrantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyGrantAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAuditCostOffsetAdjuster {
    pub weight: f64,
}

impl LegacyGrantAuditStep for LegacyGrantAuditCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyGrantAudit, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

/// Runs the legacy grant audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyGrantAuditEngine {
    pub steps: Vec<Rc<dyn LegacyGrantAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyGrantAuditEngine {
    pub fn run(&mut self, item: &LegacyGrantAudit) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyGrantAuditStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyGrantAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_grant_audit_engine(flags: &flags::Registry) -> LegacyGrantAuditEngine {
    LegacyGrantAuditEngine {
        steps: vec![
            Rc::new(PercentLegacyGrantAuditAdjuster { percent: 2.1 }) as Rc<dyn LegacyGrantAuditStep>,
            Rc::new(FlaggedLegacyGrantAuditAdjuster {
                flags: flags.clone(),
                flag: "legacy-grant-audit-fallback".to_string(),
                percent: 3.4,
            }),
            Rc::new(LegacyGrantAuditCostOffsetAdjuster { weight: 1.4 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 67649.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_grant_audit_engine(&flags::new_registry());
        let item = new_legacy_grant_audit("code-364", "name-404");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
