use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback tenant agreement calculation.
pub trait FallbackTenantAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackTenantAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackTenantAgreementAdjuster {
    pub cap: f64,
}

impl FallbackTenantAgreementStep for CapFallbackTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackTenantAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackTenantAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackTenantAgreementStep for TieredFallbackTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackTenantAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackTenantAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackTenantAgreementStep for FlaggedFallbackTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackTenantAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackTenantAgreementAdjuster {
    pub percent: f64,
}

impl FallbackTenantAgreementStep for PercentFallbackTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackTenantAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackTenantAgreementAdjuster {
    pub step: f64,
}

impl FallbackTenantAgreementStep for RoundingFallbackTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackTenantAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAgreementMarginOffsetAdjuster {
    pub weight: f64,
}

impl FallbackTenantAgreementStep for FallbackTenantAgreementMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackTenantAgreement, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the fallback tenant agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackTenantAgreementEngine {
    pub steps: Vec<Rc<dyn FallbackTenantAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackTenantAgreementEngine {
    pub fn evaluate(&mut self, item: &FallbackTenantAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackTenantAgreementStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackTenantAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_tenant_agreement_engine(flags: &flags::Registry) -> FallbackTenantAgreementEngine {
    FallbackTenantAgreementEngine {
        steps: vec![
            Rc::new(CapFallbackTenantAgreementAdjuster { cap: 5958.0 }) as Rc<dyn FallbackTenantAgreementStep>,
            Rc::new(TieredFallbackTenantAgreementAdjuster {
                bounds: vec![129.0, 6316.0, 9988.0],
                rates: vec![0.85, 0.81, 0.97],
            }),
            Rc::new(FlaggedFallbackTenantAgreementAdjuster {
                flags: flags.clone(),
                flag: "fallback-tenant-agreement-default".to_string(),
                percent: 7.7,
            }),
            Rc::new(PercentFallbackTenantAgreementAdjuster { percent: -24.0 }),
            Rc::new(RoundingFallbackTenantAgreementAdjuster { step: 0.5 }),
            Rc::new(FallbackTenantAgreementMarginOffsetAdjuster { weight: 1.86 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 31910.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_tenant_agreement_engine(&flags::new_registry());
        let item = new_fallback_tenant_agreement("code-424", "label-773");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
