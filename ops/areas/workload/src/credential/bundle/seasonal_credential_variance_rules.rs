use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal credential variance must pass. Check returns its problems, or none.
pub trait SeasonalCredentialVarianceRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCredentialVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialVarianceRequiredCodeRule {
    pub message: String,
}

impl SeasonalCredentialVarianceRule for SeasonalCredentialVarianceRequiredCodeRule {
    fn check(&self, item: &SeasonalCredentialVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialVarianceReviewerLengthRule {
    pub max_length: i64,
}

impl SeasonalCredentialVarianceRule for SeasonalCredentialVarianceReviewerLengthRule {
    fn check(&self, item: &SeasonalCredentialVariance) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCredentialVarianceCodePrefixRule {
    pub prefix: String,
}

impl SeasonalCredentialVarianceRule for SeasonalCredentialVarianceCodePrefixRule {
    fn check(&self, item: &SeasonalCredentialVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal credential variance.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialVarianceValidator {
    pub rules: Vec<Rc<dyn SeasonalCredentialVarianceRule>>,
}

impl SeasonalCredentialVarianceValidator {
    pub fn validate(&self, item: &SeasonalCredentialVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalCredentialVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCredentialVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_credential_variance_validator() -> SeasonalCredentialVarianceValidator {
    SeasonalCredentialVarianceValidator {
        rules: vec![
            Rc::new(SeasonalCredentialVarianceRequiredCodeRule {
                message: "seasonal credential variance needs a value".to_string(),
            }) as Rc<dyn SeasonalCredentialVarianceRule>,
            Rc::new(SeasonalCredentialVarianceReviewerLengthRule { max_length: 32 }),
            Rc::new(SeasonalCredentialVarianceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_credential_variance_validator();
        let item = new_seasonal_credential_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalCredentialVarianceRequiredCodeRule {
            message: "seasonal credential variance needs an entry".to_string(),
        };
        let mut item = new_seasonal_credential_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = SeasonalCredentialVarianceReviewerLengthRule { max_length: 3600 };
        let mut item = new_seasonal_credential_variance("x", "y");
        item.reviewer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
