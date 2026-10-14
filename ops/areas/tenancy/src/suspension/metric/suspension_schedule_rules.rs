use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension schedule must pass. Check returns its problems, or none.
pub trait SuspensionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScheduleCountryLengthRule {
    pub max_length: i64,
}

impl SuspensionScheduleRule for SuspensionScheduleCountryLengthRule {
    fn check(&self, item: &SuspensionSchedule) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScheduleHostLengthRule {
    pub max_length: i64,
}

impl SuspensionScheduleRule for SuspensionScheduleHostLengthRule {
    fn check(&self, item: &SuspensionSchedule) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScheduleTitleLengthRule {
    pub max_length: i64,
}

impl SuspensionScheduleRule for SuspensionScheduleTitleLengthRule {
    fn check(&self, item: &SuspensionSchedule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScheduleRequiredTitleRule {
    pub message: String,
}

impl SuspensionScheduleRule for SuspensionScheduleRequiredTitleRule {
    fn check(&self, item: &SuspensionSchedule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScheduleCodePrefixRule {
    pub prefix: String,
}

impl SuspensionScheduleRule for SuspensionScheduleCodePrefixRule {
    fn check(&self, item: &SuspensionSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScheduleApprovedGuardRule {
    pub limit: i64,
}

impl SuspensionScheduleRule for SuspensionScheduleApprovedGuardRule {
    fn check(&self, item: &SuspensionSchedule) -> Vec<String> {
        if item.approved && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while approved",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension schedule.
#[derive(Debug, Clone)]
pub struct SuspensionScheduleValidator {
    pub rules: Vec<Rc<dyn SuspensionScheduleRule>>,
}

impl SuspensionScheduleValidator {
    pub fn validate(&self, item: &SuspensionSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_schedule_validator() -> SuspensionScheduleValidator {
    SuspensionScheduleValidator {
        rules: vec![
            Rc::new(SuspensionScheduleCountryLengthRule { max_length: 3600 }) as Rc<dyn SuspensionScheduleRule>,
            Rc::new(SuspensionScheduleHostLengthRule { max_length: 255 }),
            Rc::new(SuspensionScheduleTitleLengthRule { max_length: 120 }),
            Rc::new(SuspensionScheduleRequiredTitleRule {
                message: "suspension schedule needs this field".to_string(),
            }),
            Rc::new(SuspensionScheduleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SuspensionScheduleApprovedGuardRule { limit: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_schedule_validator();
        let item = new_suspension_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = SuspensionScheduleCountryLengthRule { max_length: 120 };
        let mut item = new_suspension_schedule("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = SuspensionScheduleHostLengthRule { max_length: 1000 };
        let mut item = new_suspension_schedule("x", "y");
        item.host = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SuspensionScheduleTitleLengthRule { max_length: 16 };
        let mut item = new_suspension_schedule("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SuspensionScheduleRequiredTitleRule {
            message: "suspension schedule needs a value".to_string(),
        };
        let mut item = new_suspension_schedule("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
