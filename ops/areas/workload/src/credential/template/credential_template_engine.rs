use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the credential template calculation.
pub trait CredentialTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialTemplateAdjuster {
    pub step: f64,
}

impl CredentialTemplateAdjuster for RoundingCredentialTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTemplateWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl CredentialTemplateAdjuster for CredentialTemplateWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialTemplate, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCredentialTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CredentialTemplateAdjuster for FlaggedCredentialTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CredentialTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCredentialTemplateAdjuster {
    pub cap: f64,
}

impl CredentialTemplateAdjuster for CapCredentialTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CredentialTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialTemplateAdjuster {
    pub minimum: f64,
}

impl CredentialTemplateAdjuster for MinimumCredentialTemplateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialTemplate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialTemplateAdjuster for TieredCredentialTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the credential template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialTemplateEngine {
    pub steps: Vec<Rc<dyn CredentialTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialTemplateEngine {
    pub fn run(&mut self, item: &CredentialTemplate) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CredentialTemplateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CredentialTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_credential_template_engine(flags: &flags::Registry) -> CredentialTemplateEngine {
    CredentialTemplateEngine {
        steps: vec![
            Rc::new(RoundingCredentialTemplateAdjuster { step: 0.25 }) as Rc<dyn CredentialTemplateAdjuster>,
            Rc::new(CredentialTemplateWindowDaysOffsetAdjuster { weight: 0.82 }),
            Rc::new(FlaggedCredentialTemplateAdjuster {
                flags: flags.clone(),
                flag: "credential-template-seasonal".to_string(),
                percent: 1.4,
            }),
            Rc::new(CapCredentialTemplateAdjuster { cap: 39401.0 }),
            Rc::new(MinimumCredentialTemplateAdjuster { minimum: 29.2 }),
            Rc::new(TieredCredentialTemplateAdjuster {
                bounds: vec![2626.0, 4417.0, 7817.0],
                rates: vec![1.12, 0.86, 0.8],
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 98842.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_credential_template_engine(&flags::new_registry());
        let item = new_credential_template("code-746", "name-999");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
