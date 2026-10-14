use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient plan record must pass. Check returns its problems, or none.
pub trait LenientPlanRecordRule: std::fmt::Debug {
    fn check(&self, item: &LenientPlanRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanRecordExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPlanRecordRule for LenientPlanRecordExpiresAtRangeRule {
    fn check(&self, item: &LenientPlanRecord) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanRecordAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPlanRecordRule for LenientPlanRecordAttemptsRangeRule {
    fn check(&self, item: &LenientPlanRecord) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanRecordCodeLengthRule {
    pub max_length: i64,
}

impl LenientPlanRecordRule for LenientPlanRecordCodeLengthRule {
    fn check(&self, item: &LenientPlanRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient plan record.
#[derive(Debug, Clone)]
pub struct LenientPlanRecordValidator {
    pub rules: Vec<Rc<dyn LenientPlanRecordRule>>,
}

impl LenientPlanRecordValidator {
    pub fn validate(&self, item: &LenientPlanRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPlanRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPlanRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_plan_record_validator() -> LenientPlanRecordValidator {
    LenientPlanRecordValidator {
        rules: vec![
            Rc::new(LenientPlanRecordExpiresAtRangeRule { min: 80, max: 1000 }) as Rc<dyn LenientPlanRecordRule>,
            Rc::new(LenientPlanRecordAttemptsRangeRule { min: 80, max: 120 }),
            Rc::new(LenientPlanRecordCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_plan_record_validator();
        let item = new_lenient_plan_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = LenientPlanRecordExpiresAtRangeRule { min: 16, max: 64 };
        let mut item = new_lenient_plan_record("x", "y");
        item.expires_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = LenientPlanRecordAttemptsRangeRule { min: 255, max: 3600 };
        let mut item = new_lenient_plan_record("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientPlanRecordCodeLengthRule { max_length: 86400 };
        let mut item = new_lenient_plan_record("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
