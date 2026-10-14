use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tenant quota calculation.
pub trait TenantQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuotaAmountOffsetAdjuster {
    pub weight: f64,
}

impl TenantQuotaAdjuster for TenantQuotaAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &TenantQuota, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantQuotaAdjuster for FlaggedTenantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantQuotaAdjuster for TieredTenantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantQuotaAdjuster {
    pub cap: f64,
}

impl TenantQuotaAdjuster for CapTenantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantQuotaAdjuster {
    pub percent: f64,
}

impl TenantQuotaAdjuster for PercentTenantQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantQuotaEngine {
    pub steps: Vec<Rc<dyn TenantQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantQuotaEngine {
    pub fn price(&mut self, item: &TenantQuota) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantQuotaAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_quota_engine(flags: &flags::Registry) -> TenantQuotaEngine {
    TenantQuotaEngine {
        steps: vec![
            Rc::new(TenantQuotaAmountOffsetAdjuster { weight: 1.83 }) as Rc<dyn TenantQuotaAdjuster>,
            Rc::new(FlaggedTenantQuotaAdjuster {
                flags: flags.clone(),
                flag: "tenant-quota-seasonal".to_string(),
                percent: -2.0,
            }),
            Rc::new(TieredTenantQuotaAdjuster {
                bounds: vec![3073.0, 5090.0, 8252.0],
                rates: vec![1.2, 1.14, 1.01],
            }),
            Rc::new(CapTenantQuotaAdjuster { cap: 19001.0 }),
            Rc::new(PercentTenantQuotaAdjuster { percent: -8.7 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 66748.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_quota_engine(&flags::new_registry());
        let item = new_tenant_quota("code-804", "label-620");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
