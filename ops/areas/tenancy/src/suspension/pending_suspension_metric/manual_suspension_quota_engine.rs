use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual suspension quota calculation.
pub trait ManualSuspensionQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSuspensionQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualSuspensionQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualSuspensionQuotaFactor for FlaggedManualSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualSuspensionQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSuspensionQuotaAdjuster {
    pub cap: f64,
}

impl ManualSuspensionQuotaFactor for CapManualSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSuspensionQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualSuspensionQuotaAdjuster {
    pub minimum: f64,
}

impl ManualSuspensionQuotaFactor for MinimumManualSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualSuspensionQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualSuspensionQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualSuspensionQuotaFactor for TieredManualSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualSuspensionQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceManualSuspensionQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualSuspensionQuotaFactor for NamespaceManualSuspensionQuotaAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ManualSuspensionQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual suspension quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSuspensionQuotaEngine {
    pub steps: Vec<Rc<dyn ManualSuspensionQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSuspensionQuotaEngine {
    pub fn compute(&mut self, item: &ManualSuspensionQuota) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ManualSuspensionQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_manual_suspension_quota_engine(flags: &flags::Registry) -> ManualSuspensionQuotaEngine {
    ManualSuspensionQuotaEngine {
        steps: vec![
            Rc::new(FlaggedManualSuspensionQuotaAdjuster {
                flags: flags.clone(),
                flag: "manual-suspension-quota-weighted".to_string(),
                percent: -2.2,
            }) as Rc<dyn ManualSuspensionQuotaFactor>,
            Rc::new(CapManualSuspensionQuotaAdjuster { cap: 45752.0 }),
            Rc::new(MinimumManualSuspensionQuotaAdjuster { minimum: 45.8 }),
            Rc::new(TieredManualSuspensionQuotaAdjuster {
                bounds: vec![3557.0, 7679.0, 8727.0],
                rates: vec![1.04, 0.86, 0.89],
            }),
            Rc::new(NamespaceManualSuspensionQuotaAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.16), ("east".to_string(), 0.8)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 4241.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_suspension_quota_engine(&flags::new_registry());
        let item = new_manual_suspension_quota("code-771", "name-907");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
