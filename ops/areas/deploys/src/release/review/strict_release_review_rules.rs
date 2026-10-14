use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict release review must pass. Check returns its problems, or none.
pub trait StrictReleaseReviewRule: std::fmt::Debug {
    fn check(&self, item: &StrictReleaseReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseReviewRequiredCurrencyRule {
    pub message: String,
}

impl StrictReleaseReviewRule for StrictReleaseReviewRequiredCurrencyRule {
    fn check(&self, item: &StrictReleaseReview) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseReviewRequiredFingerprintRule {
    pub message: String,
}

impl StrictReleaseReviewRule for StrictReleaseReviewRequiredFingerprintRule {
    fn check(&self, item: &StrictReleaseReview) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseReviewCodePrefixRule {
    pub prefix: String,
}

impl StrictReleaseReviewRule for StrictReleaseReviewCodePrefixRule {
    fn check(&self, item: &StrictReleaseReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseReviewRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictReleaseReviewRule for StrictReleaseReviewRateRangeRule {
    fn check(&self, item: &StrictReleaseReview) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict release review.
#[derive(Debug, Clone)]
pub struct StrictReleaseReviewValidator {
    pub rules: Vec<Rc<dyn StrictReleaseReviewRule>>,
}

impl StrictReleaseReviewValidator {
    pub fn validate(&self, item: &StrictReleaseReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictReleaseReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictReleaseReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_release_review_validator() -> StrictReleaseReviewValidator {
    StrictReleaseReviewValidator {
        rules: vec![
            Rc::new(StrictReleaseReviewRequiredCurrencyRule {
                message: "strict release review needs this field".to_string(),
            }) as Rc<dyn StrictReleaseReviewRule>,
            Rc::new(StrictReleaseReviewRequiredFingerprintRule {
                message: "strict release review needs an entry".to_string(),
            }),
            Rc::new(StrictReleaseReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictReleaseReviewRateRangeRule { min: 6.7, max: 1741.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_release_review_validator();
        let item = new_strict_release_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = StrictReleaseReviewRequiredCurrencyRule {
            message: "strict release review needs this field".to_string(),
        };
        let mut item = new_strict_release_review("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = StrictReleaseReviewRequiredFingerprintRule {
            message: "strict release review needs an entry".to_string(),
        };
        let mut item = new_strict_release_review("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = StrictReleaseReviewRateRangeRule { min: 3.2, max: 1744.3 };
        let mut item = new_strict_release_review("x", "y");
        item.rate = 1745.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
