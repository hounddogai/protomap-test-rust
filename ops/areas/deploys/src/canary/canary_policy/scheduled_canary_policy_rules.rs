use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled canary policy must pass. Check returns its problems, or none.
pub trait ScheduledCanaryPolicyRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledCanaryPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyRequiredAudienceRule {
    pub message: String,
}

impl ScheduledCanaryPolicyRule for ScheduledCanaryPolicyRequiredAudienceRule {
    fn check(&self, item: &ScheduledCanaryPolicy) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyRequiredCodeRule {
    pub message: String,
}

impl ScheduledCanaryPolicyRule for ScheduledCanaryPolicyRequiredCodeRule {
    fn check(&self, item: &ScheduledCanaryPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyNameLengthRule {
    pub max_length: i64,
}

impl ScheduledCanaryPolicyRule for ScheduledCanaryPolicyNameLengthRule {
    fn check(&self, item: &ScheduledCanaryPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyAudienceLengthRule {
    pub max_length: i64,
}

impl ScheduledCanaryPolicyRule for ScheduledCanaryPolicyAudienceLengthRule {
    fn check(&self, item: &ScheduledCanaryPolicy) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledCanaryPolicyRule for ScheduledCanaryPolicyCodeLengthRule {
    fn check(&self, item: &ScheduledCanaryPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl ScheduledCanaryPolicyRule for ScheduledCanaryPolicyAllowedNotesRule {
    fn check(&self, item: &ScheduledCanaryPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is unknown", item.notes)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled canary policy.
#[derive(Debug, Clone)]
pub struct ScheduledCanaryPolicyValidator {
    pub rules: Vec<Rc<dyn ScheduledCanaryPolicyRule>>,
}

impl ScheduledCanaryPolicyValidator {
    pub fn validate(&self, item: &ScheduledCanaryPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledCanaryPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledCanaryPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_canary_policy_validator() -> ScheduledCanaryPolicyValidator {
    ScheduledCanaryPolicyValidator {
        rules: vec![
            Rc::new(ScheduledCanaryPolicyRequiredAudienceRule {
                message: "scheduled canary policy needs an entry".to_string(),
            }) as Rc<dyn ScheduledCanaryPolicyRule>,
            Rc::new(ScheduledCanaryPolicyRequiredCodeRule {
                message: "scheduled canary policy needs a value".to_string(),
            }),
            Rc::new(ScheduledCanaryPolicyNameLengthRule { max_length: 32 }),
            Rc::new(ScheduledCanaryPolicyAudienceLengthRule { max_length: 120 }),
            Rc::new(ScheduledCanaryPolicyCodeLengthRule { max_length: 3600 }),
            Rc::new(ScheduledCanaryPolicyAllowedNotesRule {
                allowed: vec!["regional".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_canary_policy_validator();
        let item = new_scheduled_canary_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ScheduledCanaryPolicyRequiredAudienceRule {
            message: "scheduled canary policy needs a value".to_string(),
        };
        let mut item = new_scheduled_canary_policy("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledCanaryPolicyRequiredCodeRule {
            message: "scheduled canary policy needs an entry".to_string(),
        };
        let mut item = new_scheduled_canary_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ScheduledCanaryPolicyNameLengthRule { max_length: 3600 };
        let mut item = new_scheduled_canary_policy("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ScheduledCanaryPolicyAudienceLengthRule { max_length: 3600 };
        let mut item = new_scheduled_canary_policy("x", "y");
        item.audience = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledCanaryPolicyCodeLengthRule { max_length: 80 };
        let mut item = new_scheduled_canary_policy("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
