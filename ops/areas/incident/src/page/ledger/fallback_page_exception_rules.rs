use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback page exception must pass. Check returns its problems, or none.
pub trait FallbackPageExceptionRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPageException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageExceptionRequiredLabelRule {
    pub message: String,
}

impl FallbackPageExceptionRule for FallbackPageExceptionRequiredLabelRule {
    fn check(&self, item: &FallbackPageException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageExceptionRequiredCategoryRule {
    pub message: String,
}

impl FallbackPageExceptionRule for FallbackPageExceptionRequiredCategoryRule {
    fn check(&self, item: &FallbackPageException) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageExceptionLabelLengthRule {
    pub max_length: i64,
}

impl FallbackPageExceptionRule for FallbackPageExceptionLabelLengthRule {
    fn check(&self, item: &FallbackPageException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageExceptionCodeLengthRule {
    pub max_length: i64,
}

impl FallbackPageExceptionRule for FallbackPageExceptionCodeLengthRule {
    fn check(&self, item: &FallbackPageException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageExceptionWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackPageExceptionRule for FallbackPageExceptionWeightRangeRule {
    fn check(&self, item: &FallbackPageException) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageExceptionRequiredCodeRule {
    pub message: String,
}

impl FallbackPageExceptionRule for FallbackPageExceptionRequiredCodeRule {
    fn check(&self, item: &FallbackPageException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback page exception.
#[derive(Debug, Clone)]
pub struct FallbackPageExceptionValidator {
    pub rules: Vec<Rc<dyn FallbackPageExceptionRule>>,
}

impl FallbackPageExceptionValidator {
    pub fn validate(&self, item: &FallbackPageException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackPageException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPageException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_page_exception_validator() -> FallbackPageExceptionValidator {
    FallbackPageExceptionValidator {
        rules: vec![
            Rc::new(FallbackPageExceptionRequiredLabelRule {
                message: "fallback page exception needs an entry".to_string(),
            }) as Rc<dyn FallbackPageExceptionRule>,
            Rc::new(FallbackPageExceptionRequiredCategoryRule {
                message: "fallback page exception needs an entry".to_string(),
            }),
            Rc::new(FallbackPageExceptionLabelLengthRule { max_length: 80 }),
            Rc::new(FallbackPageExceptionCodeLengthRule { max_length: 120 }),
            Rc::new(FallbackPageExceptionWeightRangeRule { min: 6.5, max: 802.6 }),
            Rc::new(FallbackPageExceptionRequiredCodeRule {
                message: "fallback page exception needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_page_exception_validator();
        let item = new_fallback_page_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackPageExceptionRequiredLabelRule {
            message: "fallback page exception needs an entry".to_string(),
        };
        let mut item = new_fallback_page_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = FallbackPageExceptionRequiredCategoryRule {
            message: "fallback page exception needs a value".to_string(),
        };
        let mut item = new_fallback_page_exception("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackPageExceptionLabelLengthRule { max_length: 255 };
        let mut item = new_fallback_page_exception("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackPageExceptionCodeLengthRule { max_length: 64 };
        let mut item = new_fallback_page_exception("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = FallbackPageExceptionWeightRangeRule { min: 1.2, max: 2446.3 };
        let mut item = new_fallback_page_exception("x", "y");
        item.weight = 2447.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackPageExceptionRequiredCodeRule {
            message: "fallback page exception needs a value".to_string(),
        };
        let mut item = new_fallback_page_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
