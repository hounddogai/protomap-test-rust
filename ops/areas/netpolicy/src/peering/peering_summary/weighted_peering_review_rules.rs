use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted peering review must pass. Check returns its problems, or none.
pub trait WeightedPeeringReviewRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPeeringReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringReviewExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPeeringReviewRule for WeightedPeeringReviewExpiresAtRangeRule {
    fn check(&self, item: &WeightedPeeringReview) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringReviewRequiredTitleRule {
    pub message: String,
}

impl WeightedPeeringReviewRule for WeightedPeeringReviewRequiredTitleRule {
    fn check(&self, item: &WeightedPeeringReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringReviewQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPeeringReviewRule for WeightedPeeringReviewQuotaUnitsRangeRule {
    fn check(&self, item: &WeightedPeeringReview) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringReviewTitleLengthRule {
    pub max_length: i64,
}

impl WeightedPeeringReviewRule for WeightedPeeringReviewTitleLengthRule {
    fn check(&self, item: &WeightedPeeringReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedPeeringReviewRule for WeightedPeeringReviewAllowedCodeRule {
    fn check(&self, item: &WeightedPeeringReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted peering review.
#[derive(Debug, Clone)]
pub struct WeightedPeeringReviewValidator {
    pub rules: Vec<Rc<dyn WeightedPeeringReviewRule>>,
}

impl WeightedPeeringReviewValidator {
    pub fn validate(&self, item: &WeightedPeeringReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedPeeringReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPeeringReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_peering_review_validator() -> WeightedPeeringReviewValidator {
    WeightedPeeringReviewValidator {
        rules: vec![
            Rc::new(WeightedPeeringReviewExpiresAtRangeRule { min: 120, max: 255 })
                as Rc<dyn WeightedPeeringReviewRule>,
            Rc::new(WeightedPeeringReviewRequiredTitleRule {
                message: "weighted peering review needs a value".to_string(),
            }),
            Rc::new(WeightedPeeringReviewQuotaUnitsRangeRule { min: 255, max: 3600 }),
            Rc::new(WeightedPeeringReviewTitleLengthRule { max_length: 3600 }),
            Rc::new(WeightedPeeringReviewAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_peering_review_validator();
        let item = new_weighted_peering_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = WeightedPeeringReviewExpiresAtRangeRule { min: 64, max: 3600 };
        let mut item = new_weighted_peering_review("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedPeeringReviewRequiredTitleRule {
            message: "weighted peering review needs a value".to_string(),
        };
        let mut item = new_weighted_peering_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = WeightedPeeringReviewQuotaUnitsRangeRule { min: 16, max: 255 };
        let mut item = new_weighted_peering_review("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedPeeringReviewTitleLengthRule { max_length: 120 };
        let mut item = new_weighted_peering_review("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
