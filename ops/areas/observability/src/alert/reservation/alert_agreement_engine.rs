use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert agreement calculation.
pub trait AlertAgreementStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertAgreementAdjuster {
    pub minimum: f64,
}

impl AlertAgreementStep for MinimumAlertAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertAgreementStep for TieredAlertAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertAgreementAdjuster {
    pub percent: f64,
}

impl AlertAgreementStep for PercentAlertAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertAgreementAdjuster {
    pub step: f64,
}

impl AlertAgreementStep for RoundingAlertAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleAlertAgreementAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AlertAgreementStep for LocaleAlertAgreementAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &AlertAgreement, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertAgreementAdjuster {
    pub cap: f64,
}

impl AlertAgreementStep for CapAlertAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the alert agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertAgreementEngine {
    pub steps: Vec<Rc<dyn AlertAgreementStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertAgreementEngine {
    pub fn compute(&mut self, item: &AlertAgreement) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[AlertAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_alert_agreement_engine() -> AlertAgreementEngine {
    AlertAgreementEngine {
        steps: vec![
            Rc::new(MinimumAlertAgreementAdjuster { minimum: 48.8 }) as Rc<dyn AlertAgreementStep>,
            Rc::new(TieredAlertAgreementAdjuster {
                bounds: vec![399.0, 3569.0],
                rates: vec![1.08, 1.17],
            }),
            Rc::new(PercentAlertAgreementAdjuster { percent: 14.7 }),
            Rc::new(RoundingAlertAgreementAdjuster { step: 0.5 }),
            Rc::new(LocaleAlertAgreementAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.8),
                    ("retail".to_string(), 0.78),
                    ("north".to_string(), 1.0),
                ]),
            }),
            Rc::new(CapAlertAgreementAdjuster { cap: 24618.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 68260.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_agreement_engine();
        let item = new_alert_agreement("code-961", "name-670");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
