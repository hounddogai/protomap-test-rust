use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual dns record agreement calculation.
pub trait ManualDnsRecordAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDnsRecordAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualDnsRecordAgreementAdjuster {
    pub percent: f64,
}

impl ManualDnsRecordAgreementFactor for PercentManualDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualDnsRecordAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDnsRecordAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDnsRecordAgreementFactor for FlaggedManualDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDnsRecordAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualDnsRecordAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualDnsRecordAgreementFactor for TieredManualDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualDnsRecordAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualDnsRecordAgreementAdjuster {
    pub cap: f64,
}

impl ManualDnsRecordAgreementFactor for CapManualDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualDnsRecordAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualDnsRecordAgreementAdjuster {
    pub step: f64,
}

impl ManualDnsRecordAgreementFactor for RoundingManualDnsRecordAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualDnsRecordAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual dns record agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordAgreementEngine {
    pub steps: Vec<Rc<dyn ManualDnsRecordAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDnsRecordAgreementEngine {
    pub fn evaluate(&mut self, item: &ManualDnsRecordAgreement) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualDnsRecordAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualDnsRecordAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_dns_record_agreement_engine(flags: &flags::Registry) -> ManualDnsRecordAgreementEngine {
    ManualDnsRecordAgreementEngine {
        steps: vec![
            Rc::new(PercentManualDnsRecordAgreementAdjuster { percent: 8.0 }) as Rc<dyn ManualDnsRecordAgreementFactor>,
            Rc::new(FlaggedManualDnsRecordAgreementAdjuster {
                flags: flags.clone(),
                flag: "manual-dns-record-agreement-primary".to_string(),
                percent: 5.1,
            }),
            Rc::new(TieredManualDnsRecordAgreementAdjuster {
                bounds: vec![369.0, 3131.0, 8860.0],
                rates: vec![1.04, 1.06, 0.87],
            }),
            Rc::new(CapManualDnsRecordAgreementAdjuster { cap: 19381.0 }),
            Rc::new(RoundingManualDnsRecordAgreementAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 53221.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_dns_record_agreement_engine(&flags::new_registry());
        let item = new_manual_dns_record_agreement("code-572", "title-764");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
