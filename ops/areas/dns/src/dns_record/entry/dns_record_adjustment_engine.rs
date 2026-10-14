use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record adjustment calculation.
pub trait DnsRecordAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordAdjustmentAdjuster {
    pub cap: f64,
}

impl DnsRecordAdjustmentModifier for CapDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitDnsRecordAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordAdjustmentModifier for UnitDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &DnsRecordAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordAdjustmentModifier for TieredDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordAdjustmentAdjuster {
    pub minimum: f64,
}

impl DnsRecordAdjustmentModifier for MinimumDnsRecordAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dns record adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordAdjustmentEngine {
    pub steps: Vec<Rc<dyn DnsRecordAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordAdjustmentEngine {
    pub fn price(&mut self, item: &DnsRecordAdjustment) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_adjustment_engine() -> DnsRecordAdjustmentEngine {
    DnsRecordAdjustmentEngine {
        steps: vec![
            Rc::new(CapDnsRecordAdjustmentAdjuster { cap: 21166.0 }) as Rc<dyn DnsRecordAdjustmentModifier>,
            Rc::new(UnitDnsRecordAdjustmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.36)]),
            }),
            Rc::new(TieredDnsRecordAdjustmentAdjuster {
                bounds: vec![2826.0, 6564.0, 6608.0, 8273.0],
                rates: vec![1.2, 0.96, 0.88, 0.99],
            }),
            Rc::new(MinimumDnsRecordAdjustmentAdjuster { minimum: 37.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 75766.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_adjustment_engine();
        let item = new_dns_record_adjustment("code-588", "name-749");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
