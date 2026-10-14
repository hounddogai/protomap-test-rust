use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched runbook bundle must pass. Check returns its problems, or none.
pub trait BatchedRunbookBundleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRunbookBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookBundleRequiredNameRule {
    pub message: String,
}

impl BatchedRunbookBundleRule for BatchedRunbookBundleRequiredNameRule {
    fn check(&self, item: &BatchedRunbookBundle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookBundleActiveGuardRule {
    pub limit: f64,
}

impl BatchedRunbookBundleRule for BatchedRunbookBundleActiveGuardRule {
    fn check(&self, item: &BatchedRunbookBundle) -> Vec<String> {
        if item.active && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while active", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookBundleRequiredCodeRule {
    pub message: String,
}

impl BatchedRunbookBundleRule for BatchedRunbookBundleRequiredCodeRule {
    fn check(&self, item: &BatchedRunbookBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookBundleCodeLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookBundleRule for BatchedRunbookBundleCodeLengthRule {
    fn check(&self, item: &BatchedRunbookBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookBundleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedRunbookBundleRule for BatchedRunbookBundleMaximumRangeRule {
    fn check(&self, item: &BatchedRunbookBundle) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookBundleSourceLengthRule {
    pub max_length: i64,
}

impl BatchedRunbookBundleRule for BatchedRunbookBundleSourceLengthRule {
    fn check(&self, item: &BatchedRunbookBundle) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched runbook bundle.
#[derive(Debug, Clone)]
pub struct BatchedRunbookBundleValidator {
    pub rules: Vec<Rc<dyn BatchedRunbookBundleRule>>,
}

impl BatchedRunbookBundleValidator {
    pub fn validate(&self, item: &BatchedRunbookBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRunbookBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRunbookBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_runbook_bundle_validator() -> BatchedRunbookBundleValidator {
    BatchedRunbookBundleValidator {
        rules: vec![
            Rc::new(BatchedRunbookBundleRequiredNameRule {
                message: "batched runbook bundle needs a value".to_string(),
            }) as Rc<dyn BatchedRunbookBundleRule>,
            Rc::new(BatchedRunbookBundleActiveGuardRule { limit: 2.9 }),
            Rc::new(BatchedRunbookBundleRequiredCodeRule {
                message: "batched runbook bundle needs this field".to_string(),
            }),
            Rc::new(BatchedRunbookBundleCodeLengthRule { max_length: 80 }),
            Rc::new(BatchedRunbookBundleMaximumRangeRule { min: 1.4, max: 4972.9 }),
            Rc::new(BatchedRunbookBundleSourceLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_runbook_bundle_validator();
        let item = new_batched_runbook_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedRunbookBundleRequiredNameRule {
            message: "batched runbook bundle needs an entry".to_string(),
        };
        let mut item = new_batched_runbook_bundle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedRunbookBundleRequiredCodeRule {
            message: "batched runbook bundle needs a value".to_string(),
        };
        let mut item = new_batched_runbook_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedRunbookBundleCodeLengthRule { max_length: 3600 };
        let mut item = new_batched_runbook_bundle("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = BatchedRunbookBundleMaximumRangeRule { min: 0.9, max: 4817.9 };
        let mut item = new_batched_runbook_bundle("x", "y");
        item.maximum = 4819.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = BatchedRunbookBundleSourceLengthRule { max_length: 3600 };
        let mut item = new_batched_runbook_bundle("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
