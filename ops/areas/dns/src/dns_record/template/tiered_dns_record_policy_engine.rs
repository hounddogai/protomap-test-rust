use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered dns record policy calculation.
pub trait TieredDnsRecordPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredDnsRecordPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredDnsRecordPolicyAdjuster {
    pub percent: f64,
}

impl TieredDnsRecordPolicyFactor for PercentTieredDnsRecordPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredDnsRecordPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordPolicyAmountOffsetAdjuster {
    pub weight: f64,
}

impl TieredDnsRecordPolicyFactor for TieredDnsRecordPolicyAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TieredDnsRecordPolicy, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredDnsRecordPolicyAdjuster {
    pub cap: f64,
}

impl TieredDnsRecordPolicyFactor for CapTieredDnsRecordPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredDnsRecordPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredDnsRecordPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredDnsRecordPolicyFactor for FlaggedTieredDnsRecordPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredDnsRecordPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredDnsRecordPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredDnsRecordPolicyFactor for TieredTieredDnsRecordPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredDnsRecordPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredDnsRecordPolicyAdjuster {
    pub minimum: f64,
}

impl TieredDnsRecordPolicyFactor for MinimumTieredDnsRecordPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredDnsRecordPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered dns record policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredDnsRecordPolicyEngine {
    pub steps: Vec<Rc<dyn TieredDnsRecordPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredDnsRecordPolicyEngine {
    pub fn price(&mut self, item: &TieredDnsRecordPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TieredDnsRecordPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_dns_record_policy_engine(flags: &flags::Registry) -> TieredDnsRecordPolicyEngine {
    TieredDnsRecordPolicyEngine {
        steps: vec![
            Rc::new(PercentTieredDnsRecordPolicyAdjuster { percent: -10.0 }) as Rc<dyn TieredDnsRecordPolicyFactor>,
            Rc::new(TieredDnsRecordPolicyAmountOffsetAdjuster { weight: 0.81 }),
            Rc::new(CapTieredDnsRecordPolicyAdjuster { cap: 22951.0 }),
            Rc::new(FlaggedTieredDnsRecordPolicyAdjuster {
                flags: flags.clone(),
                flag: "tiered-dns-record-policy-legacy".to_string(),
                percent: 9.2,
            }),
            Rc::new(TieredTieredDnsRecordPolicyAdjuster {
                bounds: vec![2571.0, 8213.0],
                rates: vec![0.86, 0.89],
            }),
            Rc::new(MinimumTieredDnsRecordPolicyAdjuster { minimum: 19.8 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 80747.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_dns_record_policy_engine(&flags::new_registry());
        let item = new_tiered_dns_record_policy("code-317", "label-906");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
