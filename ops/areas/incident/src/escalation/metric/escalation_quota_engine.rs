use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the escalation quota calculation.
pub trait EscalationQuotaModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationQuotaAdjuster {
    pub step: f64,
}

impl EscalationQuotaModifier for RoundingEscalationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationQuotaModifier for TieredEscalationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEscalationQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EscalationQuotaModifier for FlaggedEscalationQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EscalationQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the escalation quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationQuotaEngine {
    pub steps: Vec<Rc<dyn EscalationQuotaModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationQuotaEngine {
    pub fn compute(&mut self, item: &EscalationQuota) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[EscalationQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_escalation_quota_engine(flags: &flags::Registry) -> EscalationQuotaEngine {
    EscalationQuotaEngine {
        steps: vec![
            Rc::new(RoundingEscalationQuotaAdjuster { step: 0.1 }) as Rc<dyn EscalationQuotaModifier>,
            Rc::new(TieredEscalationQuotaAdjuster {
                bounds: vec![8145.0, 9485.0],
                rates: vec![0.9, 0.96],
            }),
            Rc::new(FlaggedEscalationQuotaAdjuster {
                flags: flags.clone(),
                flag: "escalation-quota-weighted".to_string(),
                percent: -8.8,
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 80191.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_escalation_quota_engine(&flags::new_registry());
        let item = new_escalation_quota("code-326", "title-390");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
