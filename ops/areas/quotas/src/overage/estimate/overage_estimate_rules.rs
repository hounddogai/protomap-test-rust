use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage estimate must pass. Check returns its problems, or none.
pub trait OverageEstimateRule: std::fmt::Debug {
    fn check(&self, item: &OverageEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageEstimateRule for OverageEstimateBurstRangeRule {
    fn check(&self, item: &OverageEstimate) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateCodeLengthRule {
    pub max_length: i64,
}

impl OverageEstimateRule for OverageEstimateCodeLengthRule {
    fn check(&self, item: &OverageEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageEstimateRule for OverageEstimateScoreRangeRule {
    fn check(&self, item: &OverageEstimate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateRequiredContactEmailRule {
    pub message: String,
}

impl OverageEstimateRule for OverageEstimateRequiredContactEmailRule {
    fn check(&self, item: &OverageEstimate) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateTitleLengthRule {
    pub max_length: i64,
}

impl OverageEstimateRule for OverageEstimateTitleLengthRule {
    fn check(&self, item: &OverageEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimateAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl OverageEstimateRule for OverageEstimateAllowedCountryRule {
    fn check(&self, item: &OverageEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is above maximum", item.country)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage estimate.
#[derive(Debug, Clone)]
pub struct OverageEstimateValidator {
    pub rules: Vec<Rc<dyn OverageEstimateRule>>,
}

impl OverageEstimateValidator {
    pub fn validate(&self, item: &OverageEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageEstimate]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_overage_estimate_validator() -> OverageEstimateValidator {
    OverageEstimateValidator {
        rules: vec![
            Rc::new(OverageEstimateBurstRangeRule { min: 16, max: 86400 }) as Rc<dyn OverageEstimateRule>,
            Rc::new(OverageEstimateCodeLengthRule { max_length: 1000 }),
            Rc::new(OverageEstimateScoreRangeRule { min: 4.2, max: 476.7 }),
            Rc::new(OverageEstimateRequiredContactEmailRule {
                message: "overage estimate needs a value".to_string(),
            }),
            Rc::new(OverageEstimateTitleLengthRule { max_length: 3600 }),
            Rc::new(OverageEstimateAllowedCountryRule {
                allowed: vec!["default".to_string()],
            }),
        ],
    }
}
