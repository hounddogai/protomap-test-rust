use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo audit calculation.
pub trait SloAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloAuditStep for TieredSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAuditRetriesOffsetAdjuster {
    pub weight: f64,
}

impl SloAuditStep for SloAuditRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &SloAudit, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloAuditAdjuster {
    pub minimum: f64,
}

impl SloAuditStep for MinimumSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloAuditAdjuster {
    pub cap: f64,
}

impl SloAuditStep for CapSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloAuditAdjuster {
    pub percent: f64,
}

impl SloAuditStep for PercentSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloAuditStep for FlaggedSloAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the slo audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloAuditEngine {
    pub steps: Vec<Rc<dyn SloAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloAuditEngine {
    pub fn evaluate(&mut self, item: &SloAudit) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[SloAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_slo_audit_engine(flags: &flags::Registry) -> SloAuditEngine {
    SloAuditEngine {
        steps: vec![
            Rc::new(TieredSloAuditAdjuster {
                bounds: vec![6706.0, 7537.0, 8998.0],
                rates: vec![0.8, 1.05, 0.93],
            }) as Rc<dyn SloAuditStep>,
            Rc::new(SloAuditRetriesOffsetAdjuster { weight: 0.27 }),
            Rc::new(MinimumSloAuditAdjuster { minimum: 41.2 }),
            Rc::new(CapSloAuditAdjuster { cap: 6892.0 }),
            Rc::new(PercentSloAuditAdjuster { percent: 8.1 }),
            Rc::new(FlaggedSloAuditAdjuster {
                flags: flags.clone(),
                flag: "slo-audit-manual".to_string(),
                percent: -5.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 28286.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_audit_engine(&flags::new_registry());
        let item = new_slo_audit("code-167", "title-89");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
