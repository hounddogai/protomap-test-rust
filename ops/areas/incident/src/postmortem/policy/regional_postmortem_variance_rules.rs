use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional postmortem variance must pass. Check returns its problems, or none.
pub trait RegionalPostmortemVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPostmortemVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemVarianceNameLengthRule {
    pub max_length: i64,
}

impl RegionalPostmortemVarianceRule for RegionalPostmortemVarianceNameLengthRule {
    fn check(&self, item: &RegionalPostmortemVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemVarianceRequiredCategoryRule {
    pub message: String,
}

impl RegionalPostmortemVarianceRule for RegionalPostmortemVarianceRequiredCategoryRule {
    fn check(&self, item: &RegionalPostmortemVariance) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemVarianceAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl RegionalPostmortemVarianceRule for RegionalPostmortemVarianceAllowedCountryRule {
    fn check(&self, item: &RegionalPostmortemVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is out of range", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemVarianceNamespaceLengthRule {
    pub max_length: i64,
}

impl RegionalPostmortemVarianceRule for RegionalPostmortemVarianceNamespaceLengthRule {
    fn check(&self, item: &RegionalPostmortemVariance) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemVarianceRequiredCodeRule {
    pub message: String,
}

impl RegionalPostmortemVarianceRule for RegionalPostmortemVarianceRequiredCodeRule {
    fn check(&self, item: &RegionalPostmortemVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional postmortem variance.
#[derive(Debug, Clone)]
pub struct RegionalPostmortemVarianceValidator {
    pub rules: Vec<Rc<dyn RegionalPostmortemVarianceRule>>,
}

impl RegionalPostmortemVarianceValidator {
    pub fn validate(&self, item: &RegionalPostmortemVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPostmortemVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPostmortemVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_postmortem_variance_validator() -> RegionalPostmortemVarianceValidator {
    RegionalPostmortemVarianceValidator {
        rules: vec![
            Rc::new(RegionalPostmortemVarianceNameLengthRule { max_length: 1000 })
                as Rc<dyn RegionalPostmortemVarianceRule>,
            Rc::new(RegionalPostmortemVarianceRequiredCategoryRule {
                message: "regional postmortem variance needs an entry".to_string(),
            }),
            Rc::new(RegionalPostmortemVarianceAllowedCountryRule {
                allowed: vec!["batched".to_string(), "primary".to_string(), "strict".to_string()],
            }),
            Rc::new(RegionalPostmortemVarianceNamespaceLengthRule { max_length: 255 }),
            Rc::new(RegionalPostmortemVarianceRequiredCodeRule {
                message: "regional postmortem variance needs this field".to_string(),
            }),
        ],
    }
}
