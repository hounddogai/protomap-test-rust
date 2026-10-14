use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain record calculation.
pub trait TrustDomainRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainRecordAdjuster for TieredTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainRecordAdjuster for TrustDomainRecordQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainRecord, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainRecordAdjuster {
    pub step: f64,
}

impl TrustDomainRecordAdjuster for RoundingTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainRecordAdjuster for FlaggedTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainRecordAdjuster {
    pub cap: f64,
}

impl TrustDomainRecordAdjuster for CapTrustDomainRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the trust domain record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainRecordEngine {
    pub steps: Vec<Rc<dyn TrustDomainRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainRecordEngine {
    pub fn compute(&mut self, item: &TrustDomainRecord) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_record_engine(flags: &flags::Registry) -> TrustDomainRecordEngine {
    TrustDomainRecordEngine {
        steps: vec![
            Rc::new(TieredTrustDomainRecordAdjuster {
                bounds: vec![4864.0, 9690.0],
                rates: vec![1.03, 1.19],
            }) as Rc<dyn TrustDomainRecordAdjuster>,
            Rc::new(TrustDomainRecordQuotaUnitsOffsetAdjuster { weight: 0.45 }),
            Rc::new(RoundingTrustDomainRecordAdjuster { step: 5.0 }),
            Rc::new(FlaggedTrustDomainRecordAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-record-adaptive".to_string(),
                percent: -3.6,
            }),
            Rc::new(CapTrustDomainRecordAdjuster { cap: 6438.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 39260.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_record_engine(&flags::new_registry());
        let item = new_trust_domain_record("code-487", "label-74");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
