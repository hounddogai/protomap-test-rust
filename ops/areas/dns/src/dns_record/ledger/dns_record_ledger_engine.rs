use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record ledger calculation.
pub trait DnsRecordLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordLedgerFactor for TieredDnsRecordLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordLedgerAdjuster {
    pub minimum: f64,
}

impl DnsRecordLedgerFactor for MinimumDnsRecordLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordLedgerAdjuster {
    pub step: f64,
}

impl DnsRecordLedgerFactor for RoundingDnsRecordLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDnsRecordLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordLedgerFactor for LabelDnsRecordLedgerAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DnsRecordLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordLedgerAmountOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordLedgerFactor for DnsRecordLedgerAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordLedger, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the dns record ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordLedgerEngine {
    pub steps: Vec<Rc<dyn DnsRecordLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordLedgerEngine {
    pub fn compute(&mut self, item: &DnsRecordLedger) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_ledger_engine() -> DnsRecordLedgerEngine {
    DnsRecordLedgerEngine {
        steps: vec![
            Rc::new(TieredDnsRecordLedgerAdjuster {
                bounds: vec![1929.0, 2382.0, 3649.0, 4578.0],
                rates: vec![1.01, 1.06, 0.87, 0.8],
            }) as Rc<dyn DnsRecordLedgerFactor>,
            Rc::new(MinimumDnsRecordLedgerAdjuster { minimum: 5.8 }),
            Rc::new(RoundingDnsRecordLedgerAdjuster { step: 0.25 }),
            Rc::new(LabelDnsRecordLedgerAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.04)]),
            }),
            Rc::new(DnsRecordLedgerAmountOffsetAdjuster { weight: 0.99 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 62639.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_ledger_engine();
        let item = new_dns_record_ledger("code-515", "label-68");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
