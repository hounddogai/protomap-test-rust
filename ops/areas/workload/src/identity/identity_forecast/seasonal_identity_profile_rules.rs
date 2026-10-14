use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal identity profile must pass. Check returns its problems, or none.
pub trait SeasonalIdentityProfileRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalIdentityProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityProfileCodePrefixRule {
    pub prefix: String,
}

impl SeasonalIdentityProfileRule for SeasonalIdentityProfileCodePrefixRule {
    fn check(&self, item: &SeasonalIdentityProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityProfileCategoryLengthRule {
    pub max_length: i64,
}

impl SeasonalIdentityProfileRule for SeasonalIdentityProfileCategoryLengthRule {
    fn check(&self, item: &SeasonalIdentityProfile) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityProfileRequiredLabelRule {
    pub message: String,
}

impl SeasonalIdentityProfileRule for SeasonalIdentityProfileRequiredLabelRule {
    fn check(&self, item: &SeasonalIdentityProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityProfileCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalIdentityProfileRule for SeasonalIdentityProfileCeilingRangeRule {
    fn check(&self, item: &SeasonalIdentityProfile) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal identity profile.
#[derive(Debug, Clone)]
pub struct SeasonalIdentityProfileValidator {
    pub rules: Vec<Rc<dyn SeasonalIdentityProfileRule>>,
}

impl SeasonalIdentityProfileValidator {
    pub fn validate(&self, item: &SeasonalIdentityProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalIdentityProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalIdentityProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_identity_profile_validator() -> SeasonalIdentityProfileValidator {
    SeasonalIdentityProfileValidator {
        rules: vec![
            Rc::new(SeasonalIdentityProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SeasonalIdentityProfileRule>,
            Rc::new(SeasonalIdentityProfileCategoryLengthRule { max_length: 1000 }),
            Rc::new(SeasonalIdentityProfileRequiredLabelRule {
                message: "seasonal identity profile needs this field".to_string(),
            }),
            Rc::new(SeasonalIdentityProfileCeilingRangeRule { min: 9.4, max: 1676.1 }),
        ],
    }
}
