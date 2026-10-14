use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary forecast score must pass. Check returns its problems, or none.
pub trait PrimaryForecastScoreRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryForecastScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastScoreCodePrefixRule {
    pub prefix: String,
}

impl PrimaryForecastScoreRule for PrimaryForecastScoreCodePrefixRule {
    fn check(&self, item: &PrimaryForecastScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastScoreRequiredCodeRule {
    pub message: String,
}

impl PrimaryForecastScoreRule for PrimaryForecastScoreRequiredCodeRule {
    fn check(&self, item: &PrimaryForecastScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastScoreRequiredLabelRule {
    pub message: String,
}

impl PrimaryForecastScoreRule for PrimaryForecastScoreRequiredLabelRule {
    fn check(&self, item: &PrimaryForecastScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary forecast score.
#[derive(Debug, Clone)]
pub struct PrimaryForecastScoreValidator {
    pub rules: Vec<Rc<dyn PrimaryForecastScoreRule>>,
}

impl PrimaryForecastScoreValidator {
    pub fn validate(&self, item: &PrimaryForecastScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryForecastScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryForecastScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_forecast_score_validator() -> PrimaryForecastScoreValidator {
    PrimaryForecastScoreValidator {
        rules: vec![
            Rc::new(PrimaryForecastScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PrimaryForecastScoreRule>,
            Rc::new(PrimaryForecastScoreRequiredCodeRule {
                message: "primary forecast score needs this field".to_string(),
            }),
            Rc::new(PrimaryForecastScoreRequiredLabelRule {
                message: "primary forecast score needs an entry".to_string(),
            }),
        ],
    }
}
