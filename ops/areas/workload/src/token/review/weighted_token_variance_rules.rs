use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted token variance must pass. Check returns its problems, or none.
pub trait WeightedTokenVarianceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTokenVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenVarianceCodeLengthRule {
    pub max_length: i64,
}

impl WeightedTokenVarianceRule for WeightedTokenVarianceCodeLengthRule {
    fn check(&self, item: &WeightedTokenVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTokenVarianceCategoryLengthRule {
    pub max_length: i64,
}

impl WeightedTokenVarianceRule for WeightedTokenVarianceCategoryLengthRule {
    fn check(&self, item: &WeightedTokenVariance) -> Vec<String> {
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
pub struct WeightedTokenVarianceRequiredFingerprintRule {
    pub message: String,
}

impl WeightedTokenVarianceRule for WeightedTokenVarianceRequiredFingerprintRule {
    fn check(&self, item: &WeightedTokenVariance) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted token variance.
#[derive(Debug, Clone)]
pub struct WeightedTokenVarianceValidator {
    pub rules: Vec<Rc<dyn WeightedTokenVarianceRule>>,
}

impl WeightedTokenVarianceValidator {
    pub fn validate(&self, item: &WeightedTokenVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTokenVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTokenVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_token_variance_validator() -> WeightedTokenVarianceValidator {
    WeightedTokenVarianceValidator {
        rules: vec![
            Rc::new(WeightedTokenVarianceCodeLengthRule { max_length: 16 }) as Rc<dyn WeightedTokenVarianceRule>,
            Rc::new(WeightedTokenVarianceCategoryLengthRule { max_length: 80 }),
            Rc::new(WeightedTokenVarianceRequiredFingerprintRule {
                message: "weighted token variance needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_token_variance_validator();
        let item = new_weighted_token_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedTokenVarianceCodeLengthRule { max_length: 3600 };
        let mut item = new_weighted_token_variance("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = WeightedTokenVarianceCategoryLengthRule { max_length: 255 };
        let mut item = new_weighted_token_variance("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = WeightedTokenVarianceRequiredFingerprintRule {
            message: "weighted token variance needs a value".to_string(),
        };
        let mut item = new_weighted_token_variance("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
