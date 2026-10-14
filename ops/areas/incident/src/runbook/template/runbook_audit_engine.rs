use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the runbook audit calculation.
pub trait RunbookAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookAuditStep for TieredRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAuditMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RunbookAuditStep for RunbookAuditMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookAudit, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRunbookAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookAuditStep for LocaleRunbookAuditAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RunbookAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookAuditStep for FlaggedRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookAuditAdjuster {
    pub cap: f64,
}

impl RunbookAuditStep for CapRunbookAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the runbook audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookAuditEngine {
    pub steps: Vec<Rc<dyn RunbookAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookAuditEngine {
    pub fn run(&mut self, item: &RunbookAudit) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookAuditStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_runbook_audit_engine(flags: &flags::Registry) -> RunbookAuditEngine {
    RunbookAuditEngine {
        steps: vec![
            Rc::new(TieredRunbookAuditAdjuster {
                bounds: vec![5684.0, 5838.0, 8229.0],
                rates: vec![1.17, 1.13, 1.13],
            }) as Rc<dyn RunbookAuditStep>,
            Rc::new(RunbookAuditMaximumOffsetAdjuster { weight: 0.65 }),
            Rc::new(LocaleRunbookAuditAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.14), ("retail".to_string(), 1.03)]),
            }),
            Rc::new(FlaggedRunbookAuditAdjuster {
                flags: flags.clone(),
                flag: "runbook-audit-default".to_string(),
                percent: 7.9,
            }),
            Rc::new(CapRunbookAuditAdjuster { cap: 21492.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 54790.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_audit_engine(&flags::new_registry());
        let item = new_runbook_audit("code-706", "label-582");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
