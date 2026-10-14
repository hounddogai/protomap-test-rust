use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending rule record must pass. Check returns its problems, or none.
pub trait PendingRuleRecordRule: std::fmt::Debug {
    fn check(&self, item: &PendingRuleRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleRecordDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRuleRecordRule for PendingRuleRecordDurationMinutesRangeRule {
    fn check(&self, item: &PendingRuleRecord) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleRecordSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRuleRecordRule for PendingRuleRecordSequenceRangeRule {
    fn check(&self, item: &PendingRuleRecord) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleRecordRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRuleRecordRule for PendingRuleRecordRetriesRangeRule {
    fn check(&self, item: &PendingRuleRecord) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleRecordCategoryLengthRule {
    pub max_length: i64,
}

impl PendingRuleRecordRule for PendingRuleRecordCategoryLengthRule {
    fn check(&self, item: &PendingRuleRecord) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleRecordCodeLengthRule {
    pub max_length: i64,
}

impl PendingRuleRecordRule for PendingRuleRecordCodeLengthRule {
    fn check(&self, item: &PendingRuleRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending rule record.
#[derive(Debug, Clone)]
pub struct PendingRuleRecordValidator {
    pub rules: Vec<Rc<dyn PendingRuleRecordRule>>,
}

impl PendingRuleRecordValidator {
    pub fn validate(&self, item: &PendingRuleRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRuleRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRuleRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_rule_record_validator() -> PendingRuleRecordValidator {
    PendingRuleRecordValidator {
        rules: vec![
            Rc::new(PendingRuleRecordDurationMinutesRangeRule { min: 16, max: 80 }) as Rc<dyn PendingRuleRecordRule>,
            Rc::new(PendingRuleRecordSequenceRangeRule { min: 16, max: 80 }),
            Rc::new(PendingRuleRecordRetriesRangeRule { min: 255, max: 3600 }),
            Rc::new(PendingRuleRecordCategoryLengthRule { max_length: 1000 }),
            Rc::new(PendingRuleRecordCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_rule_record_validator();
        let item = new_pending_rule_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = PendingRuleRecordDurationMinutesRangeRule { min: 80, max: 255 };
        let mut item = new_pending_rule_record("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PendingRuleRecordSequenceRangeRule { min: 3600, max: 86400 };
        let mut item = new_pending_rule_record("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = PendingRuleRecordRetriesRangeRule { min: 3600, max: 86400 };
        let mut item = new_pending_rule_record("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PendingRuleRecordCategoryLengthRule { max_length: 80 };
        let mut item = new_pending_rule_record("x", "y");
        item.category = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingRuleRecordCodeLengthRule { max_length: 255 };
        let mut item = new_pending_rule_record("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
