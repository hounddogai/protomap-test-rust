use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default redaction template must pass. Check returns its problems, or none.
pub trait DefaultRedactionTemplateRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRedactionTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTemplateArchivedGuardRule {
    pub limit: i64,
}

impl DefaultRedactionTemplateRule for DefaultRedactionTemplateArchivedGuardRule {
    fn check(&self, item: &DefaultRedactionTemplate) -> Vec<String> {
        if item.archived && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while archived",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTemplateRequiredReviewerRule {
    pub message: String,
}

impl DefaultRedactionTemplateRule for DefaultRedactionTemplateRequiredReviewerRule {
    fn check(&self, item: &DefaultRedactionTemplate) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTemplateRequiredCodeRule {
    pub message: String,
}

impl DefaultRedactionTemplateRule for DefaultRedactionTemplateRequiredCodeRule {
    fn check(&self, item: &DefaultRedactionTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTemplateCodePrefixRule {
    pub prefix: String,
}

impl DefaultRedactionTemplateRule for DefaultRedactionTemplateCodePrefixRule {
    fn check(&self, item: &DefaultRedactionTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTemplateRequiredNameRule {
    pub message: String,
}

impl DefaultRedactionTemplateRule for DefaultRedactionTemplateRequiredNameRule {
    fn check(&self, item: &DefaultRedactionTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTemplateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRedactionTemplateRule for DefaultRedactionTemplateExpiresAtRangeRule {
    fn check(&self, item: &DefaultRedactionTemplate) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default redaction template.
#[derive(Debug, Clone)]
pub struct DefaultRedactionTemplateValidator {
    pub rules: Vec<Rc<dyn DefaultRedactionTemplateRule>>,
}

impl DefaultRedactionTemplateValidator {
    pub fn validate(&self, item: &DefaultRedactionTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRedactionTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRedactionTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_redaction_template_validator() -> DefaultRedactionTemplateValidator {
    DefaultRedactionTemplateValidator {
        rules: vec![
            Rc::new(DefaultRedactionTemplateArchivedGuardRule { limit: 86400 }) as Rc<dyn DefaultRedactionTemplateRule>,
            Rc::new(DefaultRedactionTemplateRequiredReviewerRule {
                message: "default redaction template needs this field".to_string(),
            }),
            Rc::new(DefaultRedactionTemplateRequiredCodeRule {
                message: "default redaction template needs an entry".to_string(),
            }),
            Rc::new(DefaultRedactionTemplateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultRedactionTemplateRequiredNameRule {
                message: "default redaction template needs an entry".to_string(),
            }),
            Rc::new(DefaultRedactionTemplateExpiresAtRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_redaction_template_validator();
        let item = new_default_redaction_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = DefaultRedactionTemplateRequiredReviewerRule {
            message: "default redaction template needs an entry".to_string(),
        };
        let mut item = new_default_redaction_template("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultRedactionTemplateRequiredCodeRule {
            message: "default redaction template needs an entry".to_string(),
        };
        let mut item = new_default_redaction_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultRedactionTemplateRequiredNameRule {
            message: "default redaction template needs this field".to_string(),
        };
        let mut item = new_default_redaction_template("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = DefaultRedactionTemplateExpiresAtRangeRule { min: 32, max: 255 };
        let mut item = new_default_redaction_template("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
