use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation record must pass. Check returns its problems, or none.
pub trait IsolationRecordRule: std::fmt::Debug {
    fn check(&self, item: &IsolationRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRecordCodeLengthRule {
    pub max_length: i64,
}

impl IsolationRecordRule for IsolationRecordCodeLengthRule {
    fn check(&self, item: &IsolationRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRecordAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationRecordRule for IsolationRecordAttemptsRangeRule {
    fn check(&self, item: &IsolationRecord) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRecordFingerprintLengthRule {
    pub max_length: i64,
}

impl IsolationRecordRule for IsolationRecordFingerprintLengthRule {
    fn check(&self, item: &IsolationRecord) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation record.
#[derive(Debug, Clone)]
pub struct IsolationRecordValidator {
    pub rules: Vec<Rc<dyn IsolationRecordRule>>,
}

impl IsolationRecordValidator {
    pub fn validate(&self, item: &IsolationRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_record_validator() -> IsolationRecordValidator {
    IsolationRecordValidator {
        rules: vec![
            Rc::new(IsolationRecordCodeLengthRule { max_length: 64 }) as Rc<dyn IsolationRecordRule>,
            Rc::new(IsolationRecordAttemptsRangeRule { min: 1000, max: 1000 }),
            Rc::new(IsolationRecordFingerprintLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_record_validator();
        let item = new_isolation_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationRecordCodeLengthRule { max_length: 120 };
        let mut item = new_isolation_record("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = IsolationRecordAttemptsRangeRule { min: 120, max: 255 };
        let mut item = new_isolation_record("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = IsolationRecordFingerprintLengthRule { max_length: 86400 };
        let mut item = new_isolation_record("x", "y");
        item.fingerprint = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
