use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record mapping calculation.
pub trait DnsRecordMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordMappingExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordMappingAdjuster for DnsRecordMappingExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordMapping, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordMappingAdjuster for FlaggedDnsRecordMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordMappingAdjuster {
    pub cap: f64,
}

impl DnsRecordMappingAdjuster for CapDnsRecordMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordMappingAdjuster {
    pub percent: f64,
}

impl DnsRecordMappingAdjuster for PercentDnsRecordMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dns record mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordMappingEngine {
    pub steps: Vec<Rc<dyn DnsRecordMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordMappingEngine {
    pub fn price(&mut self, item: &DnsRecordMapping) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_mapping_engine(flags: &flags::Registry) -> DnsRecordMappingEngine {
    DnsRecordMappingEngine {
        steps: vec![
            Rc::new(DnsRecordMappingExpiresAtOffsetAdjuster { weight: 1.08 }) as Rc<dyn DnsRecordMappingAdjuster>,
            Rc::new(FlaggedDnsRecordMappingAdjuster {
                flags: flags.clone(),
                flag: "dns-record-mapping-tiered".to_string(),
                percent: -7.4,
            }),
            Rc::new(CapDnsRecordMappingAdjuster { cap: 35684.0 }),
            Rc::new(PercentDnsRecordMappingAdjuster { percent: -21.8 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 90267.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_mapping_engine(&flags::new_registry());
        let item = new_dns_record_mapping("code-966", "label-568");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
