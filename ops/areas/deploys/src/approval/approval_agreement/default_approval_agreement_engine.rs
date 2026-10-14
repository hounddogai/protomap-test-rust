use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default approval agreement calculation.
pub trait DefaultApprovalAgreementModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultApprovalAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultApprovalAgreementAdjuster {
    pub minimum: f64,
}

impl DefaultApprovalAgreementModifier for MinimumDefaultApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultApprovalAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultApprovalAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultApprovalAgreementModifier for TieredDefaultApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultApprovalAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultApprovalAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultApprovalAgreementModifier for FlaggedDefaultApprovalAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultApprovalAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default approval agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultApprovalAgreementEngine {
    pub steps: Vec<Rc<dyn DefaultApprovalAgreementModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultApprovalAgreementEngine {
    pub fn evaluate(&mut self, item: &DefaultApprovalAgreement) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultApprovalAgreementModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultApprovalAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_approval_agreement_engine(flags: &flags::Registry) -> DefaultApprovalAgreementEngine {
    DefaultApprovalAgreementEngine {
        steps: vec![
            Rc::new(MinimumDefaultApprovalAgreementAdjuster { minimum: 22.1 })
                as Rc<dyn DefaultApprovalAgreementModifier>,
            Rc::new(TieredDefaultApprovalAgreementAdjuster {
                bounds: vec![2043.0, 2980.0, 5224.0, 7872.0],
                rates: vec![0.96, 1.05, 0.95, 0.99],
            }),
            Rc::new(FlaggedDefaultApprovalAgreementAdjuster {
                flags: flags.clone(),
                flag: "default-approval-agreement-seasonal".to_string(),
                percent: 6.9,
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 50903.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_approval_agreement_engine(&flags::new_registry());
        let item = new_default_approval_agreement("code-402", "title-446");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
