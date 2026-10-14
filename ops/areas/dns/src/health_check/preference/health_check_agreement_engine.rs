use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the health check agreement calculation.
pub trait HealthCheckAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckAgreementAdjuster {
    pub percent: f64,
}

impl HealthCheckAgreementAdjuster for PercentHealthCheckAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckAgreementAdjuster {
    pub minimum: f64,
}

impl HealthCheckAgreementAdjuster for MinimumHealthCheckAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryHealthCheckAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckAgreementAdjuster for CategoryHealthCheckAgreementAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &HealthCheckAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the health check agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckAgreementEngine {
    pub steps: Vec<Rc<dyn HealthCheckAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckAgreementEngine {
    pub fn price(&mut self, item: &HealthCheckAgreement) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_health_check_agreement_engine() -> HealthCheckAgreementEngine {
    HealthCheckAgreementEngine {
        steps: vec![
            Rc::new(PercentHealthCheckAgreementAdjuster { percent: 18.0 }) as Rc<dyn HealthCheckAgreementAdjuster>,
            Rc::new(MinimumHealthCheckAgreementAdjuster { minimum: 48.4 }),
            Rc::new(CategoryHealthCheckAgreementAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.25),
                    ("east".to_string(), 1.38),
                    ("central".to_string(), 1.34),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 89938.6,
    }
}
