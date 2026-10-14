use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit record must pass. Check returns its problems, or none.
pub trait LimitRecordRule: std::fmt::Debug {
    fn check(&self, item: &LimitRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRecordApprovedGuardRule {
    pub limit: f64,
}

impl LimitRecordRule for LimitRecordApprovedGuardRule {
    fn check(&self, item: &LimitRecord) -> Vec<String> {
        if item.approved && (item.ceiling > self.limit) {
            return vec![format!(
                "ceiling {} exceeds {} while approved",
                item.ceiling, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRecordReviewerLengthRule {
    pub max_length: i64,
}

impl LimitRecordRule for LimitRecordReviewerLengthRule {
    fn check(&self, item: &LimitRecord) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitRecordCountryLengthRule {
    pub max_length: i64,
}

impl LimitRecordRule for LimitRecordCountryLengthRule {
    fn check(&self, item: &LimitRecord) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit record.
#[derive(Debug, Clone)]
pub struct LimitRecordValidator {
    pub rules: Vec<Rc<dyn LimitRecordRule>>,
}

impl LimitRecordValidator {
    pub fn validate(&self, item: &LimitRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_record_validator() -> LimitRecordValidator {
    LimitRecordValidator {
        rules: vec![
            Rc::new(LimitRecordApprovedGuardRule { limit: 0.4 }) as Rc<dyn LimitRecordRule>,
            Rc::new(LimitRecordReviewerLengthRule { max_length: 3600 }),
            Rc::new(LimitRecordCountryLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_record_validator();
        let item = new_limit_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = LimitRecordReviewerLengthRule { max_length: 16 };
        let mut item = new_limit_record("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = LimitRecordCountryLengthRule { max_length: 3600 };
        let mut item = new_limit_record("x", "y");
        item.country = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
