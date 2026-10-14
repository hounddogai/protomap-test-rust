use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record segment calculation.
pub trait DnsRecordSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordSegment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordSegmentStep for FlaggedDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordSegmentAdjuster {
    pub percent: f64,
}

impl DnsRecordSegmentStep for PercentDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSegmentRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordSegmentStep for DnsRecordSegmentRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordSegment, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordSegmentAdjuster {
    pub minimum: f64,
}

impl DnsRecordSegmentStep for MinimumDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryDnsRecordSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordSegmentStep for CountryDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &DnsRecordSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordSegmentAdjuster {
    pub step: f64,
}

impl DnsRecordSegmentStep for RoundingDnsRecordSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the dns record segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordSegmentEngine {
    pub steps: Vec<Rc<dyn DnsRecordSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordSegmentEngine {
    pub fn price(&mut self, item: &DnsRecordSegment) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_segment_engine(flags: &flags::Registry) -> DnsRecordSegmentEngine {
    DnsRecordSegmentEngine {
        steps: vec![
            Rc::new(FlaggedDnsRecordSegmentAdjuster {
                flags: flags.clone(),
                flag: "dns-record-segment-seasonal".to_string(),
                percent: 8.4,
            }) as Rc<dyn DnsRecordSegmentStep>,
            Rc::new(PercentDnsRecordSegmentAdjuster { percent: -20.5 }),
            Rc::new(DnsRecordSegmentRotationDaysOffsetAdjuster { weight: 0.84 }),
            Rc::new(MinimumDnsRecordSegmentAdjuster { minimum: 19.1 }),
            Rc::new(CountryDnsRecordSegmentAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.16),
                    ("online".to_string(), 1.06),
                    ("north".to_string(), 1.38),
                ]),
            }),
            Rc::new(RoundingDnsRecordSegmentAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 69781.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_segment_engine(&flags::new_registry());
        let item = new_dns_record_segment("code-714", "title-182");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
