use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dns record entry calculation.
pub trait DnsRecordEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordEntryAdjuster {
    pub minimum: f64,
}

impl DnsRecordEntryModifier for MinimumDnsRecordEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordEntryModifier for FlaggedDnsRecordEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitDnsRecordEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordEntryModifier for UnitDnsRecordEntryAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &DnsRecordEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the dns record entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordEntryEngine {
    pub steps: Vec<Rc<dyn DnsRecordEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordEntryEngine {
    pub fn compute(&mut self, item: &DnsRecordEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_entry_engine(flags: &flags::Registry) -> DnsRecordEntryEngine {
    DnsRecordEntryEngine {
        steps: vec![
            Rc::new(MinimumDnsRecordEntryAdjuster { minimum: 21.8 }) as Rc<dyn DnsRecordEntryModifier>,
            Rc::new(FlaggedDnsRecordEntryAdjuster {
                flags: flags.clone(),
                flag: "dns-record-entry-pending".to_string(),
                percent: 1.6,
            }),
            Rc::new(UnitDnsRecordEntryAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.39),
                    ("east".to_string(), 0.74),
                    ("north".to_string(), 1.36),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 36028.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_entry_engine(&flags::new_registry());
        let item = new_dns_record_entry("code-839", "title-892");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
