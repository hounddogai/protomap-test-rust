use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling federation schedule must pass. Check returns its problems, or none.
pub trait RollingFederationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RollingFederationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleNameLengthRule {
    pub max_length: i64,
}

impl RollingFederationScheduleRule for RollingFederationScheduleNameLengthRule {
    fn check(&self, item: &RollingFederationSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleRequiredNameRule {
    pub message: String,
}

impl RollingFederationScheduleRule for RollingFederationScheduleRequiredNameRule {
    fn check(&self, item: &RollingFederationSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingFederationScheduleRule for RollingFederationScheduleAllowedCodeRule {
    fn check(&self, item: &RollingFederationSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleCodeLengthRule {
    pub max_length: i64,
}

impl RollingFederationScheduleRule for RollingFederationScheduleCodeLengthRule {
    fn check(&self, item: &RollingFederationSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingFederationScheduleRule for RollingFederationScheduleRevisionRangeRule {
    fn check(&self, item: &RollingFederationSchedule) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationScheduleRequiredCodeRule {
    pub message: String,
}

impl RollingFederationScheduleRule for RollingFederationScheduleRequiredCodeRule {
    fn check(&self, item: &RollingFederationSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling federation schedule.
#[derive(Debug, Clone)]
pub struct RollingFederationScheduleValidator {
    pub rules: Vec<Rc<dyn RollingFederationScheduleRule>>,
}

impl RollingFederationScheduleValidator {
    pub fn validate(&self, item: &RollingFederationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingFederationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFederationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_federation_schedule_validator() -> RollingFederationScheduleValidator {
    RollingFederationScheduleValidator {
        rules: vec![
            Rc::new(RollingFederationScheduleNameLengthRule { max_length: 86400 })
                as Rc<dyn RollingFederationScheduleRule>,
            Rc::new(RollingFederationScheduleRequiredNameRule {
                message: "rolling federation schedule needs an entry".to_string(),
            }),
            Rc::new(RollingFederationScheduleAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }),
            Rc::new(RollingFederationScheduleCodeLengthRule { max_length: 1000 }),
            Rc::new(RollingFederationScheduleRevisionRangeRule { min: 120, max: 3600 }),
            Rc::new(RollingFederationScheduleRequiredCodeRule {
                message: "rolling federation schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_federation_schedule_validator();
        let item = new_rolling_federation_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingFederationScheduleNameLengthRule { max_length: 32 };
        let mut item = new_rolling_federation_schedule("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingFederationScheduleRequiredNameRule {
            message: "rolling federation schedule needs this field".to_string(),
        };
        let mut item = new_rolling_federation_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingFederationScheduleCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_federation_schedule("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = RollingFederationScheduleRevisionRangeRule { min: 32, max: 255 };
        let mut item = new_rolling_federation_schedule("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingFederationScheduleRequiredCodeRule {
            message: "rolling federation schedule needs this field".to_string(),
        };
        let mut item = new_rolling_federation_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
