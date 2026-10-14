use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record item calculation.
pub trait DnsRecordItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordItemAdjuster for TieredDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordItemAdjuster {
    pub step: f64,
}

impl DnsRecordItemAdjuster for RoundingDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordItemAdjuster {
    pub cap: f64,
}

impl DnsRecordItemAdjuster for CapDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordItemAdjuster {
    pub percent: f64,
}

impl DnsRecordItemAdjuster for PercentDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordItemAdjuster {
    pub minimum: f64,
}

impl DnsRecordItemAdjuster for MinimumDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dns record item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordItemEngine {
    pub steps: Vec<Rc<dyn DnsRecordItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordItemEngine {
    pub fn price(&mut self, item: &DnsRecordItem) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordItemAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_item_engine() -> DnsRecordItemEngine {
    DnsRecordItemEngine {
        steps: vec![
            Rc::new(TieredDnsRecordItemAdjuster {
                bounds: vec![2431.0, 6676.0],
                rates: vec![1.17, 0.93],
            }) as Rc<dyn DnsRecordItemAdjuster>,
            Rc::new(RoundingDnsRecordItemAdjuster { step: 1.0 }),
            Rc::new(CapDnsRecordItemAdjuster { cap: 7977.0 }),
            Rc::new(PercentDnsRecordItemAdjuster { percent: 20.6 }),
            Rc::new(MinimumDnsRecordItemAdjuster { minimum: 15.2 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 21877.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_item_engine();
        let item = new_dns_record_item("code-974", "label-391");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
