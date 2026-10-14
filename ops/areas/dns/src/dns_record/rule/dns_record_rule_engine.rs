use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record rule calculation.
pub trait DnsRecordRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordRuleAdjuster {
    pub minimum: f64,
}

impl DnsRecordRuleAdjuster for MinimumDnsRecordRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordRuleAdjuster for TieredDnsRecordRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordRuleAdjuster {
    pub cap: f64,
}

impl DnsRecordRuleAdjuster for CapDnsRecordRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordRuleAdjuster {
    pub step: f64,
}

impl DnsRecordRuleAdjuster for RoundingDnsRecordRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRuleLimitOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordRuleAdjuster for DnsRecordRuleLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordRule, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the dns record rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordRuleEngine {
    pub steps: Vec<Rc<dyn DnsRecordRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordRuleEngine {
    pub fn evaluate(&mut self, item: &DnsRecordRule) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dns_record_rule_engine() -> DnsRecordRuleEngine {
    DnsRecordRuleEngine {
        steps: vec![
            Rc::new(MinimumDnsRecordRuleAdjuster { minimum: 1.9 }) as Rc<dyn DnsRecordRuleAdjuster>,
            Rc::new(TieredDnsRecordRuleAdjuster {
                bounds: vec![907.0, 6119.0, 6891.0, 9190.0],
                rates: vec![1.01, 0.98, 0.81, 0.8],
            }),
            Rc::new(CapDnsRecordRuleAdjuster { cap: 48145.0 }),
            Rc::new(RoundingDnsRecordRuleAdjuster { step: 0.1 }),
            Rc::new(DnsRecordRuleLimitOffsetAdjuster { weight: 0.77 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 88451.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_rule_engine();
        let item = new_dns_record_rule("code-110", "name-863");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
