use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient tenant quota calculation.
pub trait LenientTenantQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientTenantQuota, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientTenantQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientTenantQuotaFactor for FlaggedLenientTenantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientTenantQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientTenantQuotaAdjuster {
    pub percent: f64,
}

impl LenientTenantQuotaFactor for PercentLenientTenantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientTenantQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientTenantQuotaAdjuster {
    pub step: f64,
}

impl LenientTenantQuotaFactor for RoundingLenientTenantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientTenantQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the lenient tenant quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientTenantQuotaEngine {
    pub steps: Vec<Rc<dyn LenientTenantQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientTenantQuotaEngine {
    pub fn compute(&mut self, item: &LenientTenantQuota) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientTenantQuotaFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientTenantQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_tenant_quota_engine(flags: &flags::Registry) -> LenientTenantQuotaEngine {
    LenientTenantQuotaEngine {
        steps: vec![
            Rc::new(FlaggedLenientTenantQuotaAdjuster {
                flags: flags.clone(),
                flag: "lenient-tenant-quota-default".to_string(),
                percent: 0.6,
            }) as Rc<dyn LenientTenantQuotaFactor>,
            Rc::new(PercentLenientTenantQuotaAdjuster { percent: -4.4 }),
            Rc::new(RoundingLenientTenantQuotaAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 43609.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_tenant_quota_engine(&flags::new_registry());
        let item = new_lenient_tenant_quota("code-672", "name-742");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
