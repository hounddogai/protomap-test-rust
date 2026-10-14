use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive alert quota calculation.
pub trait AdaptiveAlertQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAlertQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveAlertQuotaAdjuster {
    pub cap: f64,
}

impl AdaptiveAlertQuotaFactor for CapAdaptiveAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveAlertQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveAlertQuotaAdjuster {
    pub percent: f64,
}

impl AdaptiveAlertQuotaFactor for PercentAdaptiveAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveAlertQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveAlertQuotaAdjuster {
    pub step: f64,
}

impl AdaptiveAlertQuotaFactor for RoundingAdaptiveAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveAlertQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeAdaptiveAlertQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveAlertQuotaFactor for ScopeAdaptiveAlertQuotaAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &AdaptiveAlertQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveAlertQuotaAdjuster {
    pub minimum: f64,
}

impl AdaptiveAlertQuotaFactor for MinimumAdaptiveAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveAlertQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveAlertQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveAlertQuotaFactor for FlaggedAdaptiveAlertQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveAlertQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive alert quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAlertQuotaEngine {
    pub steps: Vec<Rc<dyn AdaptiveAlertQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAlertQuotaEngine {
    pub fn compute(&mut self, item: &AdaptiveAlertQuota) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAlertQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_alert_quota_engine(flags: &flags::Registry) -> AdaptiveAlertQuotaEngine {
    AdaptiveAlertQuotaEngine {
        steps: vec![
            Rc::new(CapAdaptiveAlertQuotaAdjuster { cap: 27414.0 }) as Rc<dyn AdaptiveAlertQuotaFactor>,
            Rc::new(PercentAdaptiveAlertQuotaAdjuster { percent: 8.0 }),
            Rc::new(RoundingAdaptiveAlertQuotaAdjuster { step: 0.05 }),
            Rc::new(ScopeAdaptiveAlertQuotaAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.3)]),
            }),
            Rc::new(MinimumAdaptiveAlertQuotaAdjuster { minimum: 34.9 }),
            Rc::new(FlaggedAdaptiveAlertQuotaAdjuster {
                flags: flags.clone(),
                flag: "adaptive-alert-quota-adaptive".to_string(),
                percent: 2.2,
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 5527.7,
    }
}
