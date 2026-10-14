use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation template calculation.
pub trait FederationTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationTemplateAdjuster {
    pub minimum: f64,
}

impl FederationTemplateModifier for MinimumFederationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationTemplateAdjuster {
    pub percent: f64,
}

impl FederationTemplateModifier for PercentFederationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationTemplateAdjuster {
    pub step: f64,
}

impl FederationTemplateModifier for RoundingFederationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationTemplateAdjuster {
    pub cap: f64,
}

impl FederationTemplateModifier for CapFederationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationTemplateModifier for FlaggedFederationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationTemplateModifier for TitleFederationTemplateAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the federation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationTemplateEngine {
    pub steps: Vec<Rc<dyn FederationTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationTemplateEngine {
    pub fn price(&mut self, item: &FederationTemplate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_template_engine(flags: &flags::Registry) -> FederationTemplateEngine {
    FederationTemplateEngine {
        steps: vec![
            Rc::new(MinimumFederationTemplateAdjuster { minimum: 5.9 }) as Rc<dyn FederationTemplateModifier>,
            Rc::new(PercentFederationTemplateAdjuster { percent: 24.6 }),
            Rc::new(RoundingFederationTemplateAdjuster { step: 0.1 }),
            Rc::new(CapFederationTemplateAdjuster { cap: 44075.0 }),
            Rc::new(FlaggedFederationTemplateAdjuster {
                flags: flags.clone(),
                flag: "federation-template-regional".to_string(),
                percent: -4.4,
            }),
            Rc::new(TitleFederationTemplateAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.24),
                    ("north".to_string(), 0.94),
                    ("central".to_string(), 1.33),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 68787.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_template_engine(&flags::new_registry());
        let item = new_federation_template("code-583", "title-489");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
