use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dns record preference calculation.
pub trait DnsRecordPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordPreferenceAdjuster {
    pub percent: f64,
}

impl DnsRecordPreferenceStep for PercentDnsRecordPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordPreferenceAdjuster {
    pub cap: f64,
}

impl DnsRecordPreferenceStep for CapDnsRecordPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordPreferenceStep for FlaggedDnsRecordPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the dns record preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordPreferenceEngine {
    pub steps: Vec<Rc<dyn DnsRecordPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordPreferenceEngine {
    pub fn compute(&mut self, item: &DnsRecordPreference) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordPreferenceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_preference_engine(flags: &flags::Registry) -> DnsRecordPreferenceEngine {
    DnsRecordPreferenceEngine {
        steps: vec![
            Rc::new(PercentDnsRecordPreferenceAdjuster { percent: -14.9 }) as Rc<dyn DnsRecordPreferenceStep>,
            Rc::new(CapDnsRecordPreferenceAdjuster { cap: 23680.0 }),
            Rc::new(FlaggedDnsRecordPreferenceAdjuster {
                flags: flags.clone(),
                flag: "dns-record-preference-lenient".to_string(),
                percent: 0.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 5304.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_preference_engine(&flags::new_registry());
        let item = new_dns_record_preference("code-868", "label-39");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
