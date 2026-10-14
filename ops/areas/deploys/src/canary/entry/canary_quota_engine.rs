use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary quota calculation.
pub trait CanaryQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryQuotaAdjuster {
    pub percent: f64,
}

impl CanaryQuotaStep for PercentCanaryQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryQuotaAdjuster {
    pub minimum: f64,
}

impl CanaryQuotaStep for MinimumCanaryQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleCanaryQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryQuotaStep for LocaleCanaryQuotaAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &CanaryQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryQuotaAdjuster {
    pub cap: f64,
}

impl CanaryQuotaStep for CapCanaryQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryQuotaStep for TieredCanaryQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the canary quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryQuotaEngine {
    pub steps: Vec<Rc<dyn CanaryQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryQuotaEngine {
    pub fn price(&mut self, item: &CanaryQuota) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[CanaryQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_quota_engine() -> CanaryQuotaEngine {
    CanaryQuotaEngine {
        steps: vec![
            Rc::new(PercentCanaryQuotaAdjuster { percent: -20.6 }) as Rc<dyn CanaryQuotaStep>,
            Rc::new(MinimumCanaryQuotaAdjuster { minimum: 8.3 }),
            Rc::new(LocaleCanaryQuotaAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.85), ("retail".to_string(), 1.08)]),
            }),
            Rc::new(CapCanaryQuotaAdjuster { cap: 48819.0 }),
            Rc::new(TieredCanaryQuotaAdjuster {
                bounds: vec![416.0, 1322.0, 2073.0],
                rates: vec![1.08, 1.01, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 41232.4,
    }
}
