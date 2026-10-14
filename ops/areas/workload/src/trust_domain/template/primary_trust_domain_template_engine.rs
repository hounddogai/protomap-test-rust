use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary trust domain template calculation.
pub trait PrimaryTrustDomainTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTrustDomainTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryTrustDomainTemplateAdjuster {
    pub cap: f64,
}

impl PrimaryTrustDomainTemplateModifier for CapPrimaryTrustDomainTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryTrustDomainTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryTrustDomainTemplateAdjuster {
    pub percent: f64,
}

impl PrimaryTrustDomainTemplateModifier for PercentPrimaryTrustDomainTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryTrustDomainTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryTrustDomainTemplateAdjuster {
    pub step: f64,
}

impl PrimaryTrustDomainTemplateModifier for RoundingPrimaryTrustDomainTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryTrustDomainTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary trust domain template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTrustDomainTemplateEngine {
    pub steps: Vec<Rc<dyn PrimaryTrustDomainTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTrustDomainTemplateEngine {
    pub fn compute(&mut self, item: &PrimaryTrustDomainTemplate) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryTrustDomainTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTrustDomainTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_trust_domain_template_engine() -> PrimaryTrustDomainTemplateEngine {
    PrimaryTrustDomainTemplateEngine {
        steps: vec![
            Rc::new(CapPrimaryTrustDomainTemplateAdjuster { cap: 23310.0 })
                as Rc<dyn PrimaryTrustDomainTemplateModifier>,
            Rc::new(PercentPrimaryTrustDomainTemplateAdjuster { percent: -4.4 }),
            Rc::new(RoundingPrimaryTrustDomainTemplateAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 84208.4,
    }
}
