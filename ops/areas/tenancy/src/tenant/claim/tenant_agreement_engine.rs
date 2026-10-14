use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tenant agreement calculation.
pub trait TenantAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantAgreementAdjuster {
    pub minimum: f64,
}

impl TenantAgreementModifier for MinimumTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantAgreementAdjuster {
    pub percent: f64,
}

impl TenantAgreementModifier for PercentTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantAgreementModifier for TieredTenantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAgreementBudgetOffsetAdjuster {
    pub weight: f64,
}

impl TenantAgreementModifier for TenantAgreementBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &TenantAgreement, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

/// Runs the tenant agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantAgreementEngine {
    pub steps: Vec<Rc<dyn TenantAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantAgreementEngine {
    pub fn run(&mut self, item: &TenantAgreement) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tenant_agreement_engine() -> TenantAgreementEngine {
    TenantAgreementEngine {
        steps: vec![
            Rc::new(MinimumTenantAgreementAdjuster { minimum: 20.0 }) as Rc<dyn TenantAgreementModifier>,
            Rc::new(PercentTenantAgreementAdjuster { percent: 17.4 }),
            Rc::new(TieredTenantAgreementAdjuster {
                bounds: vec![26.0, 5368.0, 6621.0, 6953.0],
                rates: vec![1.15, 1.07, 0.89, 1.13],
            }),
            Rc::new(TenantAgreementBudgetOffsetAdjuster { weight: 0.34 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 65668.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_agreement_engine();
        let item = new_tenant_agreement("code-138", "title-402");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
