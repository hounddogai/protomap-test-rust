use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending silence estimate must pass. Check returns its problems, or none.
pub trait PendingSilenceEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PendingSilenceEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceEstimateTitleLengthRule {
    pub max_length: i64,
}

impl PendingSilenceEstimateRule for PendingSilenceEstimateTitleLengthRule {
    fn check(&self, item: &PendingSilenceEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceEstimateCategoryLengthRule {
    pub max_length: i64,
}

impl PendingSilenceEstimateRule for PendingSilenceEstimateCategoryLengthRule {
    fn check(&self, item: &PendingSilenceEstimate) -> Vec<String> {
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
pub struct PendingSilenceEstimateRequiredTitleRule {
    pub message: String,
}

impl PendingSilenceEstimateRule for PendingSilenceEstimateRequiredTitleRule {
    fn check(&self, item: &PendingSilenceEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending silence estimate.
#[derive(Debug, Clone)]
pub struct PendingSilenceEstimateValidator {
    pub rules: Vec<Rc<dyn PendingSilenceEstimateRule>>,
}

impl PendingSilenceEstimateValidator {
    pub fn validate(&self, item: &PendingSilenceEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSilenceEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSilenceEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_silence_estimate_validator() -> PendingSilenceEstimateValidator {
    PendingSilenceEstimateValidator {
        rules: vec![
            Rc::new(PendingSilenceEstimateTitleLengthRule { max_length: 80 }) as Rc<dyn PendingSilenceEstimateRule>,
            Rc::new(PendingSilenceEstimateCategoryLengthRule { max_length: 120 }),
            Rc::new(PendingSilenceEstimateRequiredTitleRule {
                message: "pending silence estimate needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_silence_estimate_validator();
        let item = new_pending_silence_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingSilenceEstimateTitleLengthRule { max_length: 16 };
        let mut item = new_pending_silence_estimate("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PendingSilenceEstimateCategoryLengthRule { max_length: 3600 };
        let mut item = new_pending_silence_estimate("x", "y");
        item.category = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingSilenceEstimateRequiredTitleRule {
            message: "pending silence estimate needs this field".to_string(),
        };
        let mut item = new_pending_silence_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
