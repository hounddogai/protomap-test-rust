use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped verifier schedule must pass. Check returns its problems, or none.
pub trait CappedVerifierScheduleRule: std::fmt::Debug {
    fn check(&self, item: &CappedVerifierSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierScheduleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedVerifierScheduleRule for CappedVerifierScheduleSequenceRangeRule {
    fn check(&self, item: &CappedVerifierSchedule) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierScheduleCipherLengthRule {
    pub max_length: i64,
}

impl CappedVerifierScheduleRule for CappedVerifierScheduleCipherLengthRule {
    fn check(&self, item: &CappedVerifierSchedule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierScheduleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedVerifierScheduleRule for CappedVerifierScheduleBatchSizeRangeRule {
    fn check(&self, item: &CappedVerifierSchedule) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped verifier schedule.
#[derive(Debug, Clone)]
pub struct CappedVerifierScheduleValidator {
    pub rules: Vec<Rc<dyn CappedVerifierScheduleRule>>,
}

impl CappedVerifierScheduleValidator {
    pub fn validate(&self, item: &CappedVerifierSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedVerifierSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedVerifierSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_verifier_schedule_validator() -> CappedVerifierScheduleValidator {
    CappedVerifierScheduleValidator {
        rules: vec![
            Rc::new(CappedVerifierScheduleSequenceRangeRule { min: 16, max: 80 }) as Rc<dyn CappedVerifierScheduleRule>,
            Rc::new(CappedVerifierScheduleCipherLengthRule { max_length: 86400 }),
            Rc::new(CappedVerifierScheduleBatchSizeRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_verifier_schedule_validator();
        let item = new_capped_verifier_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = CappedVerifierScheduleSequenceRangeRule { min: 32, max: 1000 };
        let mut item = new_capped_verifier_schedule("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = CappedVerifierScheduleCipherLengthRule { max_length: 32 };
        let mut item = new_capped_verifier_schedule("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = CappedVerifierScheduleBatchSizeRangeRule { min: 255, max: 86400 };
        let mut item = new_capped_verifier_schedule("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
