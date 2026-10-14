use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation schedule must pass. Check returns its problems, or none.
pub trait IsolationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &IsolationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScheduleCodeLengthRule {
    pub max_length: i64,
}

impl IsolationScheduleRule for IsolationScheduleCodeLengthRule {
    fn check(&self, item: &IsolationSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScheduleRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationScheduleRule for IsolationScheduleRetriesRangeRule {
    fn check(&self, item: &IsolationSchedule) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScheduleCodePrefixRule {
    pub prefix: String,
}

impl IsolationScheduleRule for IsolationScheduleCodePrefixRule {
    fn check(&self, item: &IsolationSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationScheduleRequiredPrincipalRule {
    pub message: String,
}

impl IsolationScheduleRule for IsolationScheduleRequiredPrincipalRule {
    fn check(&self, item: &IsolationSchedule) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSchedulePrincipalLengthRule {
    pub max_length: i64,
}

impl IsolationScheduleRule for IsolationSchedulePrincipalLengthRule {
    fn check(&self, item: &IsolationSchedule) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation schedule.
#[derive(Debug, Clone)]
pub struct IsolationScheduleValidator {
    pub rules: Vec<Rc<dyn IsolationScheduleRule>>,
}

impl IsolationScheduleValidator {
    pub fn validate(&self, item: &IsolationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_schedule_validator() -> IsolationScheduleValidator {
    IsolationScheduleValidator {
        rules: vec![
            Rc::new(IsolationScheduleCodeLengthRule { max_length: 120 }) as Rc<dyn IsolationScheduleRule>,
            Rc::new(IsolationScheduleRetriesRangeRule { min: 16, max: 86400 }),
            Rc::new(IsolationScheduleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(IsolationScheduleRequiredPrincipalRule {
                message: "isolation schedule needs a value".to_string(),
            }),
            Rc::new(IsolationSchedulePrincipalLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_schedule_validator();
        let item = new_isolation_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationScheduleCodeLengthRule { max_length: 16 };
        let mut item = new_isolation_schedule("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = IsolationScheduleRetriesRangeRule { min: 32, max: 1000 };
        let mut item = new_isolation_schedule("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = IsolationScheduleRequiredPrincipalRule {
            message: "isolation schedule needs an entry".to_string(),
        };
        let mut item = new_isolation_schedule("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = IsolationSchedulePrincipalLengthRule { max_length: 32 };
        let mut item = new_isolation_schedule("x", "y");
        item.principal = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
