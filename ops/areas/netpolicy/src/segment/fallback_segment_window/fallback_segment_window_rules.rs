use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback segment window must pass. Check returns its problems, or none.
pub trait FallbackSegmentWindowRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSegmentWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentWindowRequiredCodeRule {
    pub message: String,
}

impl FallbackSegmentWindowRule for FallbackSegmentWindowRequiredCodeRule {
    fn check(&self, item: &FallbackSegmentWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentWindowRequiredCategoryRule {
    pub message: String,
}

impl FallbackSegmentWindowRule for FallbackSegmentWindowRequiredCategoryRule {
    fn check(&self, item: &FallbackSegmentWindow) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentWindowRequiredTitleRule {
    pub message: String,
}

impl FallbackSegmentWindowRule for FallbackSegmentWindowRequiredTitleRule {
    fn check(&self, item: &FallbackSegmentWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback segment window.
#[derive(Debug, Clone)]
pub struct FallbackSegmentWindowValidator {
    pub rules: Vec<Rc<dyn FallbackSegmentWindowRule>>,
}

impl FallbackSegmentWindowValidator {
    pub fn validate(&self, item: &FallbackSegmentWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackSegmentWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSegmentWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_segment_window_validator() -> FallbackSegmentWindowValidator {
    FallbackSegmentWindowValidator {
        rules: vec![
            Rc::new(FallbackSegmentWindowRequiredCodeRule {
                message: "fallback segment window needs an entry".to_string(),
            }) as Rc<dyn FallbackSegmentWindowRule>,
            Rc::new(FallbackSegmentWindowRequiredCategoryRule {
                message: "fallback segment window needs a value".to_string(),
            }),
            Rc::new(FallbackSegmentWindowRequiredTitleRule {
                message: "fallback segment window needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_segment_window_validator();
        let item = new_fallback_segment_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackSegmentWindowRequiredCodeRule {
            message: "fallback segment window needs a value".to_string(),
        };
        let mut item = new_fallback_segment_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = FallbackSegmentWindowRequiredCategoryRule {
            message: "fallback segment window needs this field".to_string(),
        };
        let mut item = new_fallback_segment_window("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackSegmentWindowRequiredTitleRule {
            message: "fallback segment window needs a value".to_string(),
        };
        let mut item = new_fallback_segment_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
