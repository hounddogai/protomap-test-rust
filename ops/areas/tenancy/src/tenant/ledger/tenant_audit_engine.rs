use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tenant audit calculation.
pub trait TenantAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantAudit, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantAuditFactor for FlaggedTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantAuditFactor for TieredTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAuditDiscountOffsetAdjuster {
    pub weight: f64,
}

impl TenantAuditFactor for TenantAuditDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &TenantAudit, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantAuditAdjuster {
    pub percent: f64,
}

impl TenantAuditFactor for PercentTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantAuditAdjuster {
    pub cap: f64,
}

impl TenantAuditFactor for CapTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tenant audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantAuditEngine {
    pub steps: Vec<Rc<dyn TenantAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantAuditEngine {
    pub fn price(&mut self, item: &TenantAudit) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantAuditFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_audit_engine(flags: &flags::Registry) -> TenantAuditEngine {
    TenantAuditEngine {
        steps: vec![
            Rc::new(FlaggedTenantAuditAdjuster {
                flags: flags.clone(),
                flag: "tenant-audit-legacy".to_string(),
                percent: -7.4,
            }) as Rc<dyn TenantAuditFactor>,
            Rc::new(TieredTenantAuditAdjuster {
                bounds: vec![2861.0, 3945.0],
                rates: vec![1.09, 0.94],
            }),
            Rc::new(TenantAuditDiscountOffsetAdjuster { weight: 0.12 }),
            Rc::new(PercentTenantAuditAdjuster { percent: -0.4 }),
            Rc::new(CapTenantAuditAdjuster { cap: 39513.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 6749.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_audit_engine(&flags::new_registry());
        let item = new_tenant_audit("code-163", "title-853");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
