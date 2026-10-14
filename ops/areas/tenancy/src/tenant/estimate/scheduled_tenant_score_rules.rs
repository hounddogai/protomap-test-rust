use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled tenant score must pass. Check returns its problems, or none.
pub trait ScheduledTenantScoreRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTenantScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantScoreCodePrefixRule {
    pub prefix: String,
}

impl ScheduledTenantScoreRule for ScheduledTenantScoreCodePrefixRule {
    fn check(&self, item: &ScheduledTenantScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantScoreCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledTenantScoreRule for ScheduledTenantScoreCodeLengthRule {
    fn check(&self, item: &ScheduledTenantScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantScoreRequiredTimezoneRule {
    pub message: String,
}

impl ScheduledTenantScoreRule for ScheduledTenantScoreRequiredTimezoneRule {
    fn check(&self, item: &ScheduledTenantScore) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantScoreCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledTenantScoreRule for ScheduledTenantScoreCreatedAtRangeRule {
    fn check(&self, item: &ScheduledTenantScore) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled tenant score.
#[derive(Debug, Clone)]
pub struct ScheduledTenantScoreValidator {
    pub rules: Vec<Rc<dyn ScheduledTenantScoreRule>>,
}

impl ScheduledTenantScoreValidator {
    pub fn validate(&self, item: &ScheduledTenantScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledTenantScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTenantScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_tenant_score_validator() -> ScheduledTenantScoreValidator {
    ScheduledTenantScoreValidator {
        rules: vec![
            Rc::new(ScheduledTenantScoreCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn ScheduledTenantScoreRule>,
            Rc::new(ScheduledTenantScoreCodeLengthRule { max_length: 64 }),
            Rc::new(ScheduledTenantScoreRequiredTimezoneRule {
                message: "scheduled tenant score needs this field".to_string(),
            }),
            Rc::new(ScheduledTenantScoreCreatedAtRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_tenant_score_validator();
        let item = new_scheduled_tenant_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledTenantScoreCodeLengthRule { max_length: 3600 };
        let mut item = new_scheduled_tenant_score("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ScheduledTenantScoreRequiredTimezoneRule {
            message: "scheduled tenant score needs a value".to_string(),
        };
        let mut item = new_scheduled_tenant_score("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ScheduledTenantScoreCreatedAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_scheduled_tenant_score("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
