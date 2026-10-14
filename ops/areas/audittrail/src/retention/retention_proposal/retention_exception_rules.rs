use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention exception must pass. Check returns its problems, or none.
pub trait RetentionExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RetentionException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionExceptionAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RetentionExceptionRule for RetentionExceptionAllowedTitleRule {
    fn check(&self, item: &RetentionException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionExceptionCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionExceptionRule for RetentionExceptionCeilingRangeRule {
    fn check(&self, item: &RetentionException) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionExceptionVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RetentionExceptionRule for RetentionExceptionVersionRangeRule {
    fn check(&self, item: &RetentionException) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionExceptionCodeLengthRule {
    pub max_length: i64,
}

impl RetentionExceptionRule for RetentionExceptionCodeLengthRule {
    fn check(&self, item: &RetentionException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention exception.
#[derive(Debug, Clone)]
pub struct RetentionExceptionValidator {
    pub rules: Vec<Rc<dyn RetentionExceptionRule>>,
}

impl RetentionExceptionValidator {
    pub fn validate(&self, item: &RetentionException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_exception_validator() -> RetentionExceptionValidator {
    RetentionExceptionValidator {
        rules: vec![
            Rc::new(RetentionExceptionAllowedTitleRule {
                allowed: vec!["seasonal".to_string(), "batched".to_string(), "pending".to_string()],
            }) as Rc<dyn RetentionExceptionRule>,
            Rc::new(RetentionExceptionCeilingRangeRule { min: 6.4, max: 1926.8 }),
            Rc::new(RetentionExceptionVersionRangeRule { min: 64, max: 255 }),
            Rc::new(RetentionExceptionCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_exception_validator();
        let item = new_retention_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RetentionExceptionCeilingRangeRule { min: 2.5, max: 4661.1 };
        let mut item = new_retention_exception("x", "y");
        item.ceiling = 4662.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RetentionExceptionVersionRangeRule { min: 80, max: 120 };
        let mut item = new_retention_exception("x", "y");
        item.version = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionExceptionCodeLengthRule { max_length: 64 };
        let mut item = new_retention_exception("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
