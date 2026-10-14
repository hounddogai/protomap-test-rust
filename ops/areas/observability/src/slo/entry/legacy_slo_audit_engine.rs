use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy slo audit calculation.
pub trait LegacySloAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacySloAudit, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacySloAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacySloAuditStep for FlaggedLegacySloAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacySloAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacySloAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacySloAuditStep for TieredLegacySloAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacySloAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacySloAuditAdjuster {
    pub minimum: f64,
}

impl LegacySloAuditStep for MinimumLegacySloAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacySloAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy slo audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacySloAuditEngine {
    pub steps: Vec<Rc<dyn LegacySloAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacySloAuditEngine {
    pub fn run(&mut self, item: &LegacySloAudit) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[LegacySloAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_slo_audit_engine(flags: &flags::Registry) -> LegacySloAuditEngine {
    LegacySloAuditEngine {
        steps: vec![
            Rc::new(FlaggedLegacySloAuditAdjuster {
                flags: flags.clone(),
                flag: "legacy-slo-audit-strict".to_string(),
                percent: 9.6,
            }) as Rc<dyn LegacySloAuditStep>,
            Rc::new(TieredLegacySloAuditAdjuster {
                bounds: vec![2507.0, 5837.0, 7108.0],
                rates: vec![1.01, 1.1, 0.86],
            }),
            Rc::new(MinimumLegacySloAuditAdjuster { minimum: 27.6 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 80358.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_slo_audit_engine(&flags::new_registry());
        let item = new_legacy_slo_audit("code-454", "label-961");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
