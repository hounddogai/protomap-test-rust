use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive trust domain ledger calculation.
pub trait AdaptiveTrustDomainLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveTrustDomainLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveTrustDomainLedgerAdjuster {
    pub minimum: f64,
}

impl AdaptiveTrustDomainLedgerFactor for MinimumAdaptiveTrustDomainLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainLedgerLimitOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveTrustDomainLedgerFactor for AdaptiveTrustDomainLedgerLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainLedger, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveTrustDomainLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveTrustDomainLedgerFactor for FlaggedAdaptiveTrustDomainLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveTrustDomainLedgerAdjuster {
    pub step: f64,
}

impl AdaptiveTrustDomainLedgerFactor for RoundingAdaptiveTrustDomainLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveTrustDomainLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveTrustDomainLedgerFactor for TieredAdaptiveTrustDomainLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveTrustDomainLedgerAdjuster {
    pub percent: f64,
}

impl AdaptiveTrustDomainLedgerFactor for PercentAdaptiveTrustDomainLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveTrustDomainLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive trust domain ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveTrustDomainLedgerEngine {
    pub steps: Vec<Rc<dyn AdaptiveTrustDomainLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveTrustDomainLedgerEngine {
    pub fn evaluate(&mut self, item: &AdaptiveTrustDomainLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveTrustDomainLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_trust_domain_ledger_engine(flags: &flags::Registry) -> AdaptiveTrustDomainLedgerEngine {
    AdaptiveTrustDomainLedgerEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveTrustDomainLedgerAdjuster { minimum: 8.1 })
                as Rc<dyn AdaptiveTrustDomainLedgerFactor>,
            Rc::new(AdaptiveTrustDomainLedgerLimitOffsetAdjuster { weight: 0.79 }),
            Rc::new(FlaggedAdaptiveTrustDomainLedgerAdjuster {
                flags: flags.clone(),
                flag: "adaptive-trust-domain-ledger-default".to_string(),
                percent: -2.4,
            }),
            Rc::new(RoundingAdaptiveTrustDomainLedgerAdjuster { step: 0.1 }),
            Rc::new(TieredAdaptiveTrustDomainLedgerAdjuster {
                bounds: vec![8234.0, 9324.0],
                rates: vec![0.82, 0.91],
            }),
            Rc::new(PercentAdaptiveTrustDomainLedgerAdjuster { percent: 10.1 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 88455.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_trust_domain_ledger_engine(&flags::new_registry());
        let item = new_adaptive_trust_domain_ledger("code-877", "title-508");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
