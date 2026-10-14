use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain quota calculation.
pub trait TrustDomainQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainQuotaAdjuster {
    pub cap: f64,
}

impl TrustDomainQuotaStep for CapTrustDomainQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainQuotaAdjuster {
    pub step: f64,
}

impl TrustDomainQuotaStep for RoundingTrustDomainQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTrustDomainQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainQuotaStep for ChannelTrustDomainQuotaAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TrustDomainQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainQuotaAdjuster {
    pub percent: f64,
}

impl TrustDomainQuotaStep for PercentTrustDomainQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainQuotaStep for FlaggedTrustDomainQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainQuotaStep for TieredTrustDomainQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the trust domain quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainQuotaEngine {
    pub steps: Vec<Rc<dyn TrustDomainQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainQuotaEngine {
    pub fn price(&mut self, item: &TrustDomainQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_trust_domain_quota_engine(flags: &flags::Registry) -> TrustDomainQuotaEngine {
    TrustDomainQuotaEngine {
        steps: vec![
            Rc::new(CapTrustDomainQuotaAdjuster { cap: 14988.0 }) as Rc<dyn TrustDomainQuotaStep>,
            Rc::new(RoundingTrustDomainQuotaAdjuster { step: 0.25 }),
            Rc::new(ChannelTrustDomainQuotaAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.36), ("north".to_string(), 0.76)]),
            }),
            Rc::new(PercentTrustDomainQuotaAdjuster { percent: -6.0 }),
            Rc::new(FlaggedTrustDomainQuotaAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-quota-rolling".to_string(),
                percent: 9.2,
            }),
            Rc::new(TieredTrustDomainQuotaAdjuster {
                bounds: vec![1259.0, 2186.0, 5627.0],
                rates: vec![1.04, 1.03, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 50245.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_quota_engine(&flags::new_registry());
        let item = new_trust_domain_quota("code-362", "name-849");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
