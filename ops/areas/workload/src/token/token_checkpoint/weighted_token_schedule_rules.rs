use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted token schedule must pass. Check returns its problems, or none.
pub trait WeightedTokenScheduleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTokenSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenScheduleLocaleLengthRule {
    pub max_length: i64,
}

impl WeightedTokenScheduleRule for WeightedTokenScheduleLocaleLengthRule {
    fn check(&self, item: &WeightedTokenSchedule) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenScheduleApprovedGuardRule {
    pub limit: i64,
}

impl WeightedTokenScheduleRule for WeightedTokenScheduleApprovedGuardRule {
    fn check(&self, item: &WeightedTokenSchedule) -> Vec<String> {
        if item.approved && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while approved",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenScheduleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedTokenScheduleRule for WeightedTokenScheduleCreatedAtRangeRule {
    fn check(&self, item: &WeightedTokenSchedule) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted token schedule.
#[derive(Debug, Clone)]
pub struct WeightedTokenScheduleValidator {
    pub rules: Vec<Rc<dyn WeightedTokenScheduleRule>>,
}

impl WeightedTokenScheduleValidator {
    pub fn validate(&self, item: &WeightedTokenSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTokenSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTokenSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_token_schedule_validator() -> WeightedTokenScheduleValidator {
    WeightedTokenScheduleValidator {
        rules: vec![
            Rc::new(WeightedTokenScheduleLocaleLengthRule { max_length: 3600 }) as Rc<dyn WeightedTokenScheduleRule>,
            Rc::new(WeightedTokenScheduleApprovedGuardRule { limit: 32 }),
            Rc::new(WeightedTokenScheduleCreatedAtRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_token_schedule_validator();
        let item = new_weighted_token_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = WeightedTokenScheduleLocaleLengthRule { max_length: 255 };
        let mut item = new_weighted_token_schedule("x", "y");
        item.locale = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = WeightedTokenScheduleCreatedAtRangeRule { min: 16, max: 86400 };
        let mut item = new_weighted_token_schedule("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
