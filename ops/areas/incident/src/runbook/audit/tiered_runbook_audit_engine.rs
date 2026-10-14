use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered runbook audit calculation.
pub trait TieredRunbookAuditModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRunbookAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditScoreOffsetAdjuster {
    pub weight: f64,
}

impl TieredRunbookAuditModifier for TieredRunbookAuditScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TieredRunbookAudit, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredRunbookAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredRunbookAuditModifier for FlaggedTieredRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredRunbookAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredRunbookAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredRunbookAuditModifier for TieredTieredRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredRunbookAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredRunbookAuditAdjuster {
    pub cap: f64,
}

impl TieredRunbookAuditModifier for CapTieredRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredRunbookAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredRunbookAuditAdjuster {
    pub minimum: f64,
}

impl TieredRunbookAuditModifier for MinimumTieredRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredRunbookAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered runbook audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRunbookAuditEngine {
    pub steps: Vec<Rc<dyn TieredRunbookAuditModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRunbookAuditEngine {
    pub fn compute(&mut self, item: &TieredRunbookAudit) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredRunbookAuditModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredRunbookAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_runbook_audit_engine(flags: &flags::Registry) -> TieredRunbookAuditEngine {
    TieredRunbookAuditEngine {
        steps: vec![
            Rc::new(TieredRunbookAuditScoreOffsetAdjuster { weight: 0.89 }) as Rc<dyn TieredRunbookAuditModifier>,
            Rc::new(FlaggedTieredRunbookAuditAdjuster {
                flags: flags.clone(),
                flag: "tiered-runbook-audit-strict".to_string(),
                percent: -7.1,
            }),
            Rc::new(TieredTieredRunbookAuditAdjuster {
                bounds: vec![554.0, 1280.0, 3081.0, 6751.0],
                rates: vec![0.93, 0.97, 1.14, 0.89],
            }),
            Rc::new(CapTieredRunbookAuditAdjuster { cap: 18235.0 }),
            Rc::new(MinimumTieredRunbookAuditAdjuster { minimum: 30.6 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 2056.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_runbook_audit_engine(&flags::new_registry());
        let item = new_tiered_runbook_audit("code-682", "name-850");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
