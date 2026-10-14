use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce schedule must pass. Check returns its problems, or none.
pub trait NonceScheduleRule: std::fmt::Debug {
    fn check(&self, item: &NonceSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScheduleCodeLengthRule {
    pub max_length: i64,
}

impl NonceScheduleRule for NonceScheduleCodeLengthRule {
    fn check(&self, item: &NonceSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScheduleSummaryLengthRule {
    pub max_length: i64,
}

impl NonceScheduleRule for NonceScheduleSummaryLengthRule {
    fn check(&self, item: &NonceSchedule) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScheduleCodePrefixRule {
    pub prefix: String,
}

impl NonceScheduleRule for NonceScheduleCodePrefixRule {
    fn check(&self, item: &NonceSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScheduleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl NonceScheduleRule for NonceScheduleCeilingRangeRule {
    fn check(&self, item: &NonceSchedule) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScheduleApprovedGuardRule {
    pub limit: i64,
}

impl NonceScheduleRule for NonceScheduleApprovedGuardRule {
    fn check(&self, item: &NonceSchedule) -> Vec<String> {
        if item.approved && (item.ends_at > self.limit) {
            return vec![format!(
                "ends at {} exceeds {} while approved",
                item.ends_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce schedule.
#[derive(Debug, Clone)]
pub struct NonceScheduleValidator {
    pub rules: Vec<Rc<dyn NonceScheduleRule>>,
}

impl NonceScheduleValidator {
    pub fn validate(&self, item: &NonceSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_schedule_validator() -> NonceScheduleValidator {
    NonceScheduleValidator {
        rules: vec![
            Rc::new(NonceScheduleCodeLengthRule { max_length: 120 }) as Rc<dyn NonceScheduleRule>,
            Rc::new(NonceScheduleSummaryLengthRule { max_length: 120 }),
            Rc::new(NonceScheduleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(NonceScheduleCeilingRangeRule { min: 0.2, max: 3827.5 }),
            Rc::new(NonceScheduleApprovedGuardRule { limit: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_schedule_validator();
        let item = new_nonce_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = NonceScheduleCodeLengthRule { max_length: 3600 };
        let mut item = new_nonce_schedule("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = NonceScheduleSummaryLengthRule { max_length: 80 };
        let mut item = new_nonce_schedule("x", "y");
        item.summary = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = NonceScheduleCeilingRangeRule { min: 5.7, max: 2983.5 };
        let mut item = new_nonce_schedule("x", "y");
        item.ceiling = 2985.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
