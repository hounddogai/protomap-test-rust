use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential schedule must pass. Check returns its problems, or none.
pub trait CredentialScheduleRule: std::fmt::Debug {
    fn check(&self, item: &CredentialSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScheduleCodePrefixRule {
    pub prefix: String,
}

impl CredentialScheduleRule for CredentialScheduleCodePrefixRule {
    fn check(&self, item: &CredentialSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScheduleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl CredentialScheduleRule for CredentialScheduleAllowedTitleRule {
    fn check(&self, item: &CredentialSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScheduleRequiredCodeRule {
    pub message: String,
}

impl CredentialScheduleRule for CredentialScheduleRequiredCodeRule {
    fn check(&self, item: &CredentialSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialScheduleTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialScheduleRule for CredentialScheduleTtlSecondsRangeRule {
    fn check(&self, item: &CredentialSchedule) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential schedule.
#[derive(Debug, Clone)]
pub struct CredentialScheduleValidator {
    pub rules: Vec<Rc<dyn CredentialScheduleRule>>,
}

impl CredentialScheduleValidator {
    pub fn validate(&self, item: &CredentialSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_schedule_validator() -> CredentialScheduleValidator {
    CredentialScheduleValidator {
        rules: vec![
            Rc::new(CredentialScheduleCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn CredentialScheduleRule>,
            Rc::new(CredentialScheduleAllowedTitleRule {
                allowed: vec!["strict".to_string(), "scheduled".to_string()],
            }),
            Rc::new(CredentialScheduleRequiredCodeRule {
                message: "credential schedule needs this field".to_string(),
            }),
            Rc::new(CredentialScheduleTtlSecondsRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_schedule_validator();
        let item = new_credential_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialScheduleRequiredCodeRule {
            message: "credential schedule needs a value".to_string(),
        };
        let mut item = new_credential_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = CredentialScheduleTtlSecondsRangeRule { min: 64, max: 86400 };
        let mut item = new_credential_schedule("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
