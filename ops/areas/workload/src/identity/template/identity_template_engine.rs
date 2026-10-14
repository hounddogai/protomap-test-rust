use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the identity template calculation.
pub trait IdentityTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityTemplateAdjuster {
    pub cap: f64,
}

impl IdentityTemplateModifier for CapIdentityTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityTemplateModifier for TieredIdentityTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityTemplateAdjuster {
    pub percent: f64,
}

impl IdentityTemplateModifier for PercentIdentityTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityTemplateModifier for FlaggedIdentityTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityTemplateEngine {
    pub steps: Vec<Rc<dyn IdentityTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityTemplateEngine {
    pub fn evaluate(&mut self, item: &IdentityTemplate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_template_engine(flags: &flags::Registry) -> IdentityTemplateEngine {
    IdentityTemplateEngine {
        steps: vec![
            Rc::new(CapIdentityTemplateAdjuster { cap: 44815.0 }) as Rc<dyn IdentityTemplateModifier>,
            Rc::new(TieredIdentityTemplateAdjuster {
                bounds: vec![562.0, 9014.0, 9147.0],
                rates: vec![1.07, 0.97, 0.82],
            }),
            Rc::new(PercentIdentityTemplateAdjuster { percent: 8.7 }),
            Rc::new(FlaggedIdentityTemplateAdjuster {
                flags: flags.clone(),
                flag: "identity-template-tiered".to_string(),
                percent: 8.9,
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 43447.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_template_engine(&flags::new_registry());
        let item = new_identity_template("code-656", "label-573");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
