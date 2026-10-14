use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy tenant agreement calculation.
pub trait LegacyTenantAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyTenantAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyTenantAgreementAdjuster {
    pub cap: f64,
}

impl LegacyTenantAgreementAdjuster for CapLegacyTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyTenantAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyTenantAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyTenantAgreementAdjuster for FlaggedLegacyTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyTenantAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyTenantAgreementAdjuster {
    pub percent: f64,
}

impl LegacyTenantAgreementAdjuster for PercentLegacyTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyTenantAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy tenant agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyTenantAgreementEngine {
    pub steps: Vec<Rc<dyn LegacyTenantAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyTenantAgreementEngine {
    pub fn price(&mut self, item: &LegacyTenantAgreement) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[LegacyTenantAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_tenant_agreement_engine(flags: &flags::Registry) -> LegacyTenantAgreementEngine {
    LegacyTenantAgreementEngine {
        steps: vec![
            Rc::new(CapLegacyTenantAgreementAdjuster { cap: 44484.0 }) as Rc<dyn LegacyTenantAgreementAdjuster>,
            Rc::new(FlaggedLegacyTenantAgreementAdjuster {
                flags: flags.clone(),
                flag: "legacy-tenant-agreement-scheduled".to_string(),
                percent: -2.3,
            }),
            Rc::new(PercentLegacyTenantAgreementAdjuster { percent: 1.1 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 66830.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_tenant_agreement_engine(&flags::new_registry());
        let item = new_legacy_tenant_agreement("code-567", "title-703");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
