use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dns record profile calculation.
pub trait DnsRecordProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordProfileAdjuster {
    pub percent: f64,
}

impl DnsRecordProfileAdjuster for PercentDnsRecordProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordProfileAdjuster for TieredDnsRecordProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProfileAmountOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordProfileAdjuster for DnsRecordProfileAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordProfile, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDnsRecordProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordProfileAdjuster for RegionDnsRecordProfileAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &DnsRecordProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the dns record profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordProfileEngine {
    pub steps: Vec<Rc<dyn DnsRecordProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordProfileEngine {
    pub fn compute(&mut self, item: &DnsRecordProfile) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_profile_engine() -> DnsRecordProfileEngine {
    DnsRecordProfileEngine {
        steps: vec![
            Rc::new(PercentDnsRecordProfileAdjuster { percent: -16.9 }) as Rc<dyn DnsRecordProfileAdjuster>,
            Rc::new(TieredDnsRecordProfileAdjuster {
                bounds: vec![1946.0, 2644.0, 3856.0, 9505.0],
                rates: vec![1.04, 1.15, 1.07, 1.08],
            }),
            Rc::new(DnsRecordProfileAmountOffsetAdjuster { weight: 0.11 }),
            Rc::new(RegionDnsRecordProfileAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.93)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 86817.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_profile_engine();
        let item = new_dns_record_profile("code-233", "label-761");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
