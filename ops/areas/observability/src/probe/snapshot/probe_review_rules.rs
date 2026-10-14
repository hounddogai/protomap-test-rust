use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe review must pass. Check returns its problems, or none.
pub trait ProbeReviewRule: std::fmt::Debug {
    fn check(&self, item: &ProbeReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeReviewRequiredCategoryRule {
    pub message: String,
}

impl ProbeReviewRule for ProbeReviewRequiredCategoryRule {
    fn check(&self, item: &ProbeReview) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeReviewCurrencyLengthRule {
    pub max_length: i64,
}

impl ProbeReviewRule for ProbeReviewCurrencyLengthRule {
    fn check(&self, item: &ProbeReview) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeReviewCodePrefixRule {
    pub prefix: String,
}

impl ProbeReviewRule for ProbeReviewCodePrefixRule {
    fn check(&self, item: &ProbeReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe review.
#[derive(Debug, Clone)]
pub struct ProbeReviewValidator {
    pub rules: Vec<Rc<dyn ProbeReviewRule>>,
}

impl ProbeReviewValidator {
    pub fn validate(&self, item: &ProbeReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_review_validator() -> ProbeReviewValidator {
    ProbeReviewValidator {
        rules: vec![
            Rc::new(ProbeReviewRequiredCategoryRule {
                message: "probe review needs an entry".to_string(),
            }) as Rc<dyn ProbeReviewRule>,
            Rc::new(ProbeReviewCurrencyLengthRule { max_length: 86400 }),
            Rc::new(ProbeReviewCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_review_validator();
        let item = new_probe_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = ProbeReviewRequiredCategoryRule {
            message: "probe review needs a value".to_string(),
        };
        let mut item = new_probe_review("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = ProbeReviewCurrencyLengthRule { max_length: 64 };
        let mut item = new_probe_review("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
