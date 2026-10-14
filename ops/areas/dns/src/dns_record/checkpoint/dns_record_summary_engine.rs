use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the dns record summary calculation.
pub trait DnsRecordSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryDnsRecordSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordSummaryAdjuster for CategoryDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &DnsRecordSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDnsRecordSummaryAdjuster {
    pub step: f64,
}

impl DnsRecordSummaryAdjuster for RoundingDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DnsRecordSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordSummaryAdjuster {
    pub minimum: f64,
}

impl DnsRecordSummaryAdjuster for MinimumDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDnsRecordSummaryAdjuster {
    pub cap: f64,
}

impl DnsRecordSummaryAdjuster for CapDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DnsRecordSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordSummaryAdjuster for FlaggedDnsRecordSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSummaryDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordSummaryAdjuster for DnsRecordSummaryDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordSummary, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

/// Runs the dns record summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordSummaryEngine {
    pub steps: Vec<Rc<dyn DnsRecordSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordSummaryEngine {
    pub fn price(&mut self, item: &DnsRecordSummary) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dns_record_summary_engine(flags: &flags::Registry) -> DnsRecordSummaryEngine {
    DnsRecordSummaryEngine {
        steps: vec![
            Rc::new(CategoryDnsRecordSummaryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.27)]),
            }) as Rc<dyn DnsRecordSummaryAdjuster>,
            Rc::new(RoundingDnsRecordSummaryAdjuster { step: 1.0 }),
            Rc::new(MinimumDnsRecordSummaryAdjuster { minimum: 32.9 }),
            Rc::new(CapDnsRecordSummaryAdjuster { cap: 34041.0 }),
            Rc::new(FlaggedDnsRecordSummaryAdjuster {
                flags: flags.clone(),
                flag: "dns-record-summary-tiered".to_string(),
                percent: -1.4,
            }),
            Rc::new(DnsRecordSummaryDurationMinutesOffsetAdjuster { weight: 0.68 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 37382.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_summary_engine(&flags::new_registry());
        let item = new_dns_record_summary("code-170", "title-644");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
