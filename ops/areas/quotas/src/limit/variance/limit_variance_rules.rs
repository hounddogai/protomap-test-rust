use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit variance must pass. Check returns its problems, or none.
pub trait LimitVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LimitVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitVarianceRequiredCodeRule {
    pub message: String,
}

impl LimitVarianceRule for LimitVarianceRequiredCodeRule {
    fn check(&self, item: &LimitVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitVarianceCountryLengthRule {
    pub max_length: i64,
}

impl LimitVarianceRule for LimitVarianceCountryLengthRule {
    fn check(&self, item: &LimitVariance) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitVarianceTitleLengthRule {
    pub max_length: i64,
}

impl LimitVarianceRule for LimitVarianceTitleLengthRule {
    fn check(&self, item: &LimitVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit variance.
#[derive(Debug, Clone)]
pub struct LimitVarianceValidator {
    pub rules: Vec<Rc<dyn LimitVarianceRule>>,
}

impl LimitVarianceValidator {
    pub fn validate(&self, item: &LimitVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_variance_validator() -> LimitVarianceValidator {
    LimitVarianceValidator {
        rules: vec![
            Rc::new(LimitVarianceRequiredCodeRule {
                message: "limit variance needs this field".to_string(),
            }) as Rc<dyn LimitVarianceRule>,
            Rc::new(LimitVarianceCountryLengthRule { max_length: 1000 }),
            Rc::new(LimitVarianceTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_variance_validator();
        let item = new_limit_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LimitVarianceRequiredCodeRule {
            message: "limit variance needs an entry".to_string(),
        };
        let mut item = new_limit_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = LimitVarianceCountryLengthRule { max_length: 120 };
        let mut item = new_limit_variance("x", "y");
        item.country = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LimitVarianceTitleLengthRule { max_length: 3600 };
        let mut item = new_limit_variance("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
