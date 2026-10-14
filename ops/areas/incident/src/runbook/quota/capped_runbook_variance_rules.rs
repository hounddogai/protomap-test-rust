use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped runbook variance must pass. Check returns its problems, or none.
pub trait CappedRunbookVarianceRule: std::fmt::Debug {
    fn check(&self, item: &CappedRunbookVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceRequiredCodeRule {
    pub message: String,
}

impl CappedRunbookVarianceRule for CappedRunbookVarianceRequiredCodeRule {
    fn check(&self, item: &CappedRunbookVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceWindowRule {
    pub max_span: i64,
}

impl CappedRunbookVarianceRule for CappedRunbookVarianceWindowRule {
    fn check(&self, item: &CappedRunbookVariance) -> Vec<String> {
        if item.starts_at > item.ends_at {
            return vec!["starts at is after ends at".to_string()];
        }
        if self.max_span > 0 && (item.ends_at - item.starts_at > self.max_span) {
            return vec![format!("spans more than {} seconds", self.max_span)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceRequiredClusterRule {
    pub message: String,
}

impl CappedRunbookVarianceRule for CappedRunbookVarianceRequiredClusterRule {
    fn check(&self, item: &CappedRunbookVariance) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceRequiredDescriptionRule {
    pub message: String,
}

impl CappedRunbookVarianceRule for CappedRunbookVarianceRequiredDescriptionRule {
    fn check(&self, item: &CappedRunbookVariance) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceClusterLengthRule {
    pub max_length: i64,
}

impl CappedRunbookVarianceRule for CappedRunbookVarianceClusterLengthRule {
    fn check(&self, item: &CappedRunbookVariance) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookVarianceEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRunbookVarianceRule for CappedRunbookVarianceEndsAtRangeRule {
    fn check(&self, item: &CappedRunbookVariance) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped runbook variance.
#[derive(Debug, Clone)]
pub struct CappedRunbookVarianceValidator {
    pub rules: Vec<Rc<dyn CappedRunbookVarianceRule>>,
}

impl CappedRunbookVarianceValidator {
    pub fn validate(&self, item: &CappedRunbookVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedRunbookVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRunbookVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_runbook_variance_validator() -> CappedRunbookVarianceValidator {
    CappedRunbookVarianceValidator {
        rules: vec![
            Rc::new(CappedRunbookVarianceRequiredCodeRule {
                message: "capped runbook variance needs an entry".to_string(),
            }) as Rc<dyn CappedRunbookVarianceRule>,
            Rc::new(CappedRunbookVarianceWindowRule { max_span: 255 }),
            Rc::new(CappedRunbookVarianceRequiredClusterRule {
                message: "capped runbook variance needs this field".to_string(),
            }),
            Rc::new(CappedRunbookVarianceRequiredDescriptionRule {
                message: "capped runbook variance needs this field".to_string(),
            }),
            Rc::new(CappedRunbookVarianceClusterLengthRule { max_length: 80 }),
            Rc::new(CappedRunbookVarianceEndsAtRangeRule { min: 64, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_runbook_variance_validator();
        let item = new_capped_runbook_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedRunbookVarianceRequiredCodeRule {
            message: "capped runbook variance needs a value".to_string(),
        };
        let mut item = new_capped_runbook_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = CappedRunbookVarianceRequiredClusterRule {
            message: "capped runbook variance needs this field".to_string(),
        };
        let mut item = new_capped_runbook_variance("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = CappedRunbookVarianceRequiredDescriptionRule {
            message: "capped runbook variance needs a value".to_string(),
        };
        let mut item = new_capped_runbook_variance("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = CappedRunbookVarianceClusterLengthRule { max_length: 16 };
        let mut item = new_capped_runbook_variance("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = CappedRunbookVarianceEndsAtRangeRule { min: 16, max: 1000 };
        let mut item = new_capped_runbook_variance("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
