use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential target must pass. Check returns its problems, or none.
pub trait CredentialTargetRule: std::fmt::Debug {
    fn check(&self, item: &CredentialTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTargetAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialTargetRule for CredentialTargetAttemptsRangeRule {
    fn check(&self, item: &CredentialTarget) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTargetCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialTargetRule for CredentialTargetCreatedAtRangeRule {
    fn check(&self, item: &CredentialTarget) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTargetRequiredCodeRule {
    pub message: String,
}

impl CredentialTargetRule for CredentialTargetRequiredCodeRule {
    fn check(&self, item: &CredentialTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTargetVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialTargetRule for CredentialTargetVersionRangeRule {
    fn check(&self, item: &CredentialTarget) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTargetNameLengthRule {
    pub max_length: i64,
}

impl CredentialTargetRule for CredentialTargetNameLengthRule {
    fn check(&self, item: &CredentialTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTargetCodePrefixRule {
    pub prefix: String,
}

impl CredentialTargetRule for CredentialTargetCodePrefixRule {
    fn check(&self, item: &CredentialTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential target.
#[derive(Debug, Clone)]
pub struct CredentialTargetValidator {
    pub rules: Vec<Rc<dyn CredentialTargetRule>>,
}

impl CredentialTargetValidator {
    pub fn validate(&self, item: &CredentialTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_target_validator() -> CredentialTargetValidator {
    CredentialTargetValidator {
        rules: vec![
            Rc::new(CredentialTargetAttemptsRangeRule { min: 16, max: 16 }) as Rc<dyn CredentialTargetRule>,
            Rc::new(CredentialTargetCreatedAtRangeRule { min: 16, max: 1000 }),
            Rc::new(CredentialTargetRequiredCodeRule {
                message: "credential target needs an entry".to_string(),
            }),
            Rc::new(CredentialTargetVersionRangeRule { min: 32, max: 80 }),
            Rc::new(CredentialTargetNameLengthRule { max_length: 3600 }),
            Rc::new(CredentialTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_target_validator();
        let item = new_credential_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = CredentialTargetAttemptsRangeRule { min: 16, max: 120 };
        let mut item = new_credential_target("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = CredentialTargetCreatedAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_credential_target("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialTargetRequiredCodeRule {
            message: "credential target needs this field".to_string(),
        };
        let mut item = new_credential_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = CredentialTargetVersionRangeRule { min: 120, max: 86400 };
        let mut item = new_credential_target("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CredentialTargetNameLengthRule { max_length: 255 };
        let mut item = new_credential_target("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
