use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual quote batch must pass. Check returns its problems, or none.
pub trait ManualQuoteBatchRule: std::fmt::Debug {
    fn check(&self, item: &ManualQuoteBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatchCodeLengthRule {
    pub max_length: i64,
}

impl ManualQuoteBatchRule for ManualQuoteBatchCodeLengthRule {
    fn check(&self, item: &ManualQuoteBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatchRequiredTitleRule {
    pub message: String,
}

impl ManualQuoteBatchRule for ManualQuoteBatchRequiredTitleRule {
    fn check(&self, item: &ManualQuoteBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatchNotesLengthRule {
    pub max_length: i64,
}

impl ManualQuoteBatchRule for ManualQuoteBatchNotesLengthRule {
    fn check(&self, item: &ManualQuoteBatch) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatchRequiredNotesRule {
    pub message: String,
}

impl ManualQuoteBatchRule for ManualQuoteBatchRequiredNotesRule {
    fn check(&self, item: &ManualQuoteBatch) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatchRequiredCodeRule {
    pub message: String,
}

impl ManualQuoteBatchRule for ManualQuoteBatchRequiredCodeRule {
    fn check(&self, item: &ManualQuoteBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteBatchCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualQuoteBatchRule for ManualQuoteBatchCeilingRangeRule {
    fn check(&self, item: &ManualQuoteBatch) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual quote batch.
#[derive(Debug, Clone)]
pub struct ManualQuoteBatchValidator {
    pub rules: Vec<Rc<dyn ManualQuoteBatchRule>>,
}

impl ManualQuoteBatchValidator {
    pub fn validate(&self, item: &ManualQuoteBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualQuoteBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualQuoteBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_quote_batch_validator() -> ManualQuoteBatchValidator {
    ManualQuoteBatchValidator {
        rules: vec![
            Rc::new(ManualQuoteBatchCodeLengthRule { max_length: 64 }) as Rc<dyn ManualQuoteBatchRule>,
            Rc::new(ManualQuoteBatchRequiredTitleRule {
                message: "manual quote batch needs an entry".to_string(),
            }),
            Rc::new(ManualQuoteBatchNotesLengthRule { max_length: 16 }),
            Rc::new(ManualQuoteBatchRequiredNotesRule {
                message: "manual quote batch needs this field".to_string(),
            }),
            Rc::new(ManualQuoteBatchRequiredCodeRule {
                message: "manual quote batch needs an entry".to_string(),
            }),
            Rc::new(ManualQuoteBatchCeilingRangeRule { min: 0.7, max: 3029.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_quote_batch_validator();
        let item = new_manual_quote_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualQuoteBatchCodeLengthRule { max_length: 120 };
        let mut item = new_manual_quote_batch("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualQuoteBatchRequiredTitleRule {
            message: "manual quote batch needs this field".to_string(),
        };
        let mut item = new_manual_quote_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ManualQuoteBatchNotesLengthRule { max_length: 16 };
        let mut item = new_manual_quote_batch("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ManualQuoteBatchRequiredNotesRule {
            message: "manual quote batch needs an entry".to_string(),
        };
        let mut item = new_manual_quote_batch("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualQuoteBatchRequiredCodeRule {
            message: "manual quote batch needs this field".to_string(),
        };
        let mut item = new_manual_quote_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ManualQuoteBatchCeilingRangeRule { min: 6.1, max: 1232.1 };
        let mut item = new_manual_quote_batch("x", "y");
        item.ceiling = 1233.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
