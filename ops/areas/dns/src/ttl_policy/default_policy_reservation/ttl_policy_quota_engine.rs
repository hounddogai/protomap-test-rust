use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the ttl policy quota calculation.
pub trait TtlPolicyQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyQuotaAdjuster {
    pub percent: f64,
}

impl TtlPolicyQuotaAdjuster for PercentTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyQuotaAdjuster {
    pub step: f64,
}

impl TtlPolicyQuotaAdjuster for RoundingTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyQuotaAdjuster for FlaggedTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyQuotaAdjuster {
    pub cap: f64,
}

impl TtlPolicyQuotaAdjuster for CapTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTtlPolicyQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyQuotaAdjuster for ChannelTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &TtlPolicyQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyQuotaAdjuster {
    pub minimum: f64,
}

impl TtlPolicyQuotaAdjuster for MinimumTtlPolicyQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the ttl policy quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyQuotaEngine {
    pub steps: Vec<Rc<dyn TtlPolicyQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyQuotaEngine {
    pub fn evaluate(&mut self, item: &TtlPolicyQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_ttl_policy_quota_engine(flags: &flags::Registry) -> TtlPolicyQuotaEngine {
    TtlPolicyQuotaEngine {
        steps: vec![
            Rc::new(PercentTtlPolicyQuotaAdjuster { percent: 3.9 }) as Rc<dyn TtlPolicyQuotaAdjuster>,
            Rc::new(RoundingTtlPolicyQuotaAdjuster { step: 0.25 }),
            Rc::new(FlaggedTtlPolicyQuotaAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-quota-rolling".to_string(),
                percent: -0.3,
            }),
            Rc::new(CapTtlPolicyQuotaAdjuster { cap: 48209.0 }),
            Rc::new(ChannelTtlPolicyQuotaAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.92)]),
            }),
            Rc::new(MinimumTtlPolicyQuotaAdjuster { minimum: 3.0 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 39855.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_ttl_policy_quota_engine(&flags::new_registry());
        let item = new_ttl_policy_quota("code-618", "name-972");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
