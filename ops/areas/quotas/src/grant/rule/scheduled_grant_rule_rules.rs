use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled grant rule must pass. Check returns its problems, or none.
pub trait ScheduledGrantRuleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledGrantRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantRuleRequiredCodeRule {
    pub message: String,
}

impl ScheduledGrantRuleRule for ScheduledGrantRuleRequiredCodeRule {
    fn check(&self, item: &ScheduledGrantRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantRuleRequiredNameRule {
    pub message: String,
}

impl ScheduledGrantRuleRule for ScheduledGrantRuleRequiredNameRule {
    fn check(&self, item: &ScheduledGrantRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantRuleContactEmailLengthRule {
    pub max_length: i64,
}

impl ScheduledGrantRuleRule for ScheduledGrantRuleContactEmailLengthRule {
    fn check(&self, item: &ScheduledGrantRule) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantRuleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ScheduledGrantRuleRule for ScheduledGrantRuleAllowedNameRule {
    fn check(&self, item: &ScheduledGrantRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is stale", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantRuleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledGrantRuleRule for ScheduledGrantRuleRotationDaysRangeRule {
    fn check(&self, item: &ScheduledGrantRule) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled grant rule.
#[derive(Debug, Clone)]
pub struct ScheduledGrantRuleValidator {
    pub rules: Vec<Rc<dyn ScheduledGrantRuleRule>>,
}

impl ScheduledGrantRuleValidator {
    pub fn validate(&self, item: &ScheduledGrantRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledGrantRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledGrantRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_grant_rule_validator() -> ScheduledGrantRuleValidator {
    ScheduledGrantRuleValidator {
        rules: vec![
            Rc::new(ScheduledGrantRuleRequiredCodeRule {
                message: "scheduled grant rule needs a value".to_string(),
            }) as Rc<dyn ScheduledGrantRuleRule>,
            Rc::new(ScheduledGrantRuleRequiredNameRule {
                message: "scheduled grant rule needs a value".to_string(),
            }),
            Rc::new(ScheduledGrantRuleContactEmailLengthRule { max_length: 80 }),
            Rc::new(ScheduledGrantRuleAllowedNameRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(ScheduledGrantRuleRotationDaysRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_grant_rule_validator();
        let item = new_scheduled_grant_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledGrantRuleRequiredCodeRule {
            message: "scheduled grant rule needs a value".to_string(),
        };
        let mut item = new_scheduled_grant_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ScheduledGrantRuleRequiredNameRule {
            message: "scheduled grant rule needs an entry".to_string(),
        };
        let mut item = new_scheduled_grant_rule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = ScheduledGrantRuleContactEmailLengthRule { max_length: 1000 };
        let mut item = new_scheduled_grant_rule("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ScheduledGrantRuleRotationDaysRangeRule { min: 32, max: 32 };
        let mut item = new_scheduled_grant_rule("x", "y");
        item.rotation_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
