use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dns record agreement calculation.
pub trait DnsRecordAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DnsRecordAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDnsRecordAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DnsRecordAgreementAdjuster for LabelDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DnsRecordAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DnsRecordAgreementAdjuster for TieredDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DnsRecordAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDnsRecordAgreementAdjuster {
    pub minimum: f64,
}

impl DnsRecordAgreementAdjuster for MinimumDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DnsRecordAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDnsRecordAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DnsRecordAgreementAdjuster for FlaggedDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DnsRecordAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDnsRecordAgreementAdjuster {
    pub percent: f64,
}

impl DnsRecordAgreementAdjuster for PercentDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DnsRecordAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAgreementSequenceOffsetAdjuster {
    pub weight: f64,
}

impl DnsRecordAgreementAdjuster for DnsRecordAgreementSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &DnsRecordAgreement, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the dns record agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DnsRecordAgreementEngine {
    pub steps: Vec<Rc<dyn DnsRecordAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DnsRecordAgreementEngine {
    pub fn evaluate(&mut self, item: &DnsRecordAgreement) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DnsRecordAgreementAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DnsRecordAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dns_record_agreement_engine(flags: &flags::Registry) -> DnsRecordAgreementEngine {
    DnsRecordAgreementEngine {
        steps: vec![
            Rc::new(LabelDnsRecordAgreementAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.32),
                    ("east".to_string(), 0.98),
                    ("retail".to_string(), 0.9),
                ]),
            }) as Rc<dyn DnsRecordAgreementAdjuster>,
            Rc::new(TieredDnsRecordAgreementAdjuster {
                bounds: vec![2049.0, 4607.0, 6981.0],
                rates: vec![1.15, 0.86, 1.11],
            }),
            Rc::new(MinimumDnsRecordAgreementAdjuster { minimum: 45.3 }),
            Rc::new(FlaggedDnsRecordAgreementAdjuster {
                flags: flags.clone(),
                flag: "dns-record-agreement-fallback".to_string(),
                percent: 1.8,
            }),
            Rc::new(PercentDnsRecordAgreementAdjuster { percent: 15.4 }),
            Rc::new(DnsRecordAgreementSequenceOffsetAdjuster { weight: 0.21 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 7231.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_dns_record_agreement_engine(&flags::new_registry());
        let item = new_dns_record_agreement("code-882", "label-674");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
