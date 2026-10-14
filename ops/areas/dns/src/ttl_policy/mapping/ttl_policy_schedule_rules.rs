use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy schedule must pass. Check returns its problems, or none.
pub trait TtlPolicyScheduleRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScheduleRequiredCurrencyRule {
    pub message: String,
}

impl TtlPolicyScheduleRule for TtlPolicyScheduleRequiredCurrencyRule {
    fn check(&self, item: &TtlPolicySchedule) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScheduleCodeLengthRule {
    pub max_length: i64,
}

impl TtlPolicyScheduleRule for TtlPolicyScheduleCodeLengthRule {
    fn check(&self, item: &TtlPolicySchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScheduleTitleLengthRule {
    pub max_length: i64,
}

impl TtlPolicyScheduleRule for TtlPolicyScheduleTitleLengthRule {
    fn check(&self, item: &TtlPolicySchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScheduleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyScheduleRule for TtlPolicyScheduleAllowedNameRule {
    fn check(&self, item: &TtlPolicySchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyScheduleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyScheduleRule for TtlPolicyScheduleAttemptsRangeRule {
    fn check(&self, item: &TtlPolicySchedule) -> Vec<String> {
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
pub struct TtlPolicyScheduleRequiredTitleRule {
    pub message: String,
}

impl TtlPolicyScheduleRule for TtlPolicyScheduleRequiredTitleRule {
    fn check(&self, item: &TtlPolicySchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy schedule.
#[derive(Debug, Clone)]
pub struct TtlPolicyScheduleValidator {
    pub rules: Vec<Rc<dyn TtlPolicyScheduleRule>>,
}

impl TtlPolicyScheduleValidator {
    pub fn validate(&self, item: &TtlPolicySchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_schedule_validator() -> TtlPolicyScheduleValidator {
    TtlPolicyScheduleValidator {
        rules: vec![
            Rc::new(TtlPolicyScheduleRequiredCurrencyRule {
                message: "ttl policy schedule needs an entry".to_string(),
            }) as Rc<dyn TtlPolicyScheduleRule>,
            Rc::new(TtlPolicyScheduleCodeLengthRule { max_length: 3600 }),
            Rc::new(TtlPolicyScheduleTitleLengthRule { max_length: 80 }),
            Rc::new(TtlPolicyScheduleAllowedNameRule {
                allowed: vec!["rolling".to_string(), "batched".to_string()],
            }),
            Rc::new(TtlPolicyScheduleAttemptsRangeRule { min: 120, max: 255 }),
            Rc::new(TtlPolicyScheduleRequiredTitleRule {
                message: "ttl policy schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_schedule_validator();
        let item = new_ttl_policy_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = TtlPolicyScheduleRequiredCurrencyRule {
            message: "ttl policy schedule needs a value".to_string(),
        };
        let mut item = new_ttl_policy_schedule("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TtlPolicyScheduleCodeLengthRule { max_length: 255 };
        let mut item = new_ttl_policy_schedule("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TtlPolicyScheduleTitleLengthRule { max_length: 3600 };
        let mut item = new_ttl_policy_schedule("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TtlPolicyScheduleAttemptsRangeRule { min: 16, max: 80 };
        let mut item = new_ttl_policy_schedule("x", "y");
        item.attempts = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TtlPolicyScheduleRequiredTitleRule {
            message: "ttl policy schedule needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
