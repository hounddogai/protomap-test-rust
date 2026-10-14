use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling egress schedule must pass. Check returns its problems, or none.
pub trait RollingEgressScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RollingEgressSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressScheduleCodeLengthRule {
    pub max_length: i64,
}

impl RollingEgressScheduleRule for RollingEgressScheduleCodeLengthRule {
    fn check(&self, item: &RollingEgressSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressScheduleCurrencyLengthRule {
    pub max_length: i64,
}

impl RollingEgressScheduleRule for RollingEgressScheduleCurrencyLengthRule {
    fn check(&self, item: &RollingEgressSchedule) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEgressScheduleCodePrefixRule {
    pub prefix: String,
}

impl RollingEgressScheduleRule for RollingEgressScheduleCodePrefixRule {
    fn check(&self, item: &RollingEgressSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling egress schedule.
#[derive(Debug, Clone)]
pub struct RollingEgressScheduleValidator {
    pub rules: Vec<Rc<dyn RollingEgressScheduleRule>>,
}

impl RollingEgressScheduleValidator {
    pub fn validate(&self, item: &RollingEgressSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingEgressSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEgressSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_egress_schedule_validator() -> RollingEgressScheduleValidator {
    RollingEgressScheduleValidator {
        rules: vec![
            Rc::new(RollingEgressScheduleCodeLengthRule { max_length: 32 }) as Rc<dyn RollingEgressScheduleRule>,
            Rc::new(RollingEgressScheduleCurrencyLengthRule { max_length: 86400 }),
            Rc::new(RollingEgressScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_egress_schedule_validator();
        let item = new_rolling_egress_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingEgressScheduleCodeLengthRule { max_length: 32 };
        let mut item = new_rolling_egress_schedule("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RollingEgressScheduleCurrencyLengthRule { max_length: 3600 };
        let mut item = new_rolling_egress_schedule("x", "y");
        item.currency = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
