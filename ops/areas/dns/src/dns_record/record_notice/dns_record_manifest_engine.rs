use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record manifest calculation.
pub trait DnsRecordManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordManifest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordManifestStep for FlaggedDnsRecordManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDnsRecordManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordManifestStep for NamespaceDnsRecordManifestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &DnsRecordManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordManifestAdjuster {
    pub minimum: f64,
}

impl DnsRecordManifestStep for MinimumDnsRecordManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the dns record manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordManifestEngine {
    pub steps: Vec<Rc<dyn DnsRecordManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordManifestEngine {
    pub fn price(&mut self, item: &DnsRecordManifest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_manifest_engine(flags: &flags::Registry) -> DnsRecordManifestEngine {
    DnsRecordManifestEngine {
        steps: vec![
            Rc::new(FlaggedDnsRecordManifestAdjuster {
                flags: flags.clone(),
                flag: "dns-record-manifest-seasonal".to_string(),
                percent: -5.8,
            }) as Rc<dyn DnsRecordManifestStep>,
            Rc::new(NamespaceDnsRecordManifestAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.09),
                    ("west".to_string(), 1.31),
                    ("east".to_string(), 0.78),
                ]),
            }),
            Rc::new(MinimumDnsRecordManifestAdjuster { minimum: 28.2 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 61873.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_manifest_engine(&flags::new_registry());
        let item = new_dns_record_manifest("code-337", "name-64");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
