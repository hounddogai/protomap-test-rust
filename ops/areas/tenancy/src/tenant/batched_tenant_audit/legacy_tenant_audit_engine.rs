use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy tenant audit calculation.
pub trait LegacyTenantAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyTenantAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyTenantAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyTenantAuditAdjuster for TieredLegacyTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyTenantAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLegacyTenantAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyTenantAuditAdjuster for SegmentLegacyTenantAuditAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &LegacyTenantAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyTenantAuditAdjuster {
    pub cap: f64,
}

impl LegacyTenantAuditAdjuster for CapLegacyTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyTenantAudit, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyTenantAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyTenantAuditAdjuster for FlaggedLegacyTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyTenantAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyTenantAuditAdjuster {
    pub step: f64,
}

impl LegacyTenantAuditAdjuster for RoundingLegacyTenantAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyTenantAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy tenant audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyTenantAuditEngine {
    pub steps: Vec<Rc<dyn LegacyTenantAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyTenantAuditEngine {
    pub fn run(&mut self, item: &LegacyTenantAudit) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyTenantAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyTenantAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_tenant_audit_engine(flags: &flags::Registry) -> LegacyTenantAuditEngine {
    LegacyTenantAuditEngine {
        steps: vec![
            Rc::new(TieredLegacyTenantAuditAdjuster {
                bounds: vec![2946.0, 5573.0, 7411.0, 8092.0],
                rates: vec![1.12, 1.16, 1.2, 0.95],
            }) as Rc<dyn LegacyTenantAuditAdjuster>,
            Rc::new(SegmentLegacyTenantAuditAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.03)]),
            }),
            Rc::new(CapLegacyTenantAuditAdjuster { cap: 34385.0 }),
            Rc::new(FlaggedLegacyTenantAuditAdjuster {
                flags: flags.clone(),
                flag: "legacy-tenant-audit-global".to_string(),
                percent: -6.1,
            }),
            Rc::new(RoundingLegacyTenantAuditAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 30852.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_tenant_audit_engine(&flags::new_registry());
        let item = new_legacy_tenant_audit("code-46", "title-249");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
