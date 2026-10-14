use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record threshold calculation.
pub trait DnsRecordThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDnsRecordThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordThresholdStep for TitleDnsRecordThresholdAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DnsRecordThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordThresholdStep for FlaggedDnsRecordThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordThresholdStep for TieredDnsRecordThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThresholdGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordThresholdStep for DnsRecordThresholdGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordThreshold, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the dns record threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordThresholdEngine {
    pub steps: Vec<Rc<dyn DnsRecordThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordThresholdEngine {
    pub fn compute(&mut self, item: &DnsRecordThreshold) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_dns_record_threshold_engine(flags: &flags::Registry) -> DnsRecordThresholdEngine {
    DnsRecordThresholdEngine {
        steps: vec![
            Rc::new(TitleDnsRecordThresholdAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.11),
                    ("retail".to_string(), 0.75),
                    ("central".to_string(), 1.04),
                ]),
            }) as Rc<dyn DnsRecordThresholdStep>,
            Rc::new(FlaggedDnsRecordThresholdAdjuster {
                flags: flags.clone(),
                flag: "dns-record-threshold-scheduled".to_string(),
                percent: -4.9,
            }),
            Rc::new(TieredDnsRecordThresholdAdjuster {
                bounds: vec![894.0, 3552.0, 8290.0],
                rates: vec![1.0, 0.84, 0.87],
            }),
            Rc::new(DnsRecordThresholdGraceDaysOffsetAdjuster { weight: 0.04 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 15528.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_threshold_engine(&flags::new_registry());
        let item = new_dns_record_threshold("code-747", "title-661");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
