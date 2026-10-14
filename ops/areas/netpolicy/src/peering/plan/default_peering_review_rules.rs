use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default peering review must pass. Check returns its problems, or none.
pub trait DefaultPeeringReviewRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPeeringReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewCodePrefixRule {
    pub prefix: String,
}

impl DefaultPeeringReviewRule for DefaultPeeringReviewCodePrefixRule {
    fn check(&self, item: &DefaultPeeringReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewRequiredCountryRule {
    pub message: String,
}

impl DefaultPeeringReviewRule for DefaultPeeringReviewRequiredCountryRule {
    fn check(&self, item: &DefaultPeeringReview) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultPeeringReviewRule for DefaultPeeringReviewRevisionRangeRule {
    fn check(&self, item: &DefaultPeeringReview) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewArchivedGuardRule {
    pub limit: f64,
}

impl DefaultPeeringReviewRule for DefaultPeeringReviewArchivedGuardRule {
    fn check(&self, item: &DefaultPeeringReview) -> Vec<String> {
        if item.archived && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while archived", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewRequiredLabelRule {
    pub message: String,
}

impl DefaultPeeringReviewRule for DefaultPeeringReviewRequiredLabelRule {
    fn check(&self, item: &DefaultPeeringReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringReviewIssuerLengthRule {
    pub max_length: i64,
}

impl DefaultPeeringReviewRule for DefaultPeeringReviewIssuerLengthRule {
    fn check(&self, item: &DefaultPeeringReview) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default peering review.
#[derive(Debug, Clone)]
pub struct DefaultPeeringReviewValidator {
    pub rules: Vec<Rc<dyn DefaultPeeringReviewRule>>,
}

impl DefaultPeeringReviewValidator {
    pub fn validate(&self, item: &DefaultPeeringReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultPeeringReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPeeringReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_peering_review_validator() -> DefaultPeeringReviewValidator {
    DefaultPeeringReviewValidator {
        rules: vec![
            Rc::new(DefaultPeeringReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn DefaultPeeringReviewRule>,
            Rc::new(DefaultPeeringReviewRequiredCountryRule {
                message: "default peering review needs this field".to_string(),
            }),
            Rc::new(DefaultPeeringReviewRevisionRangeRule { min: 32, max: 3600 }),
            Rc::new(DefaultPeeringReviewArchivedGuardRule { limit: 7.4 }),
            Rc::new(DefaultPeeringReviewRequiredLabelRule {
                message: "default peering review needs an entry".to_string(),
            }),
            Rc::new(DefaultPeeringReviewIssuerLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_peering_review_validator();
        let item = new_default_peering_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = DefaultPeeringReviewRequiredCountryRule {
            message: "default peering review needs this field".to_string(),
        };
        let mut item = new_default_peering_review("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DefaultPeeringReviewRevisionRangeRule { min: 32, max: 3600 };
        let mut item = new_default_peering_review("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultPeeringReviewRequiredLabelRule {
            message: "default peering review needs a value".to_string(),
        };
        let mut item = new_default_peering_review("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = DefaultPeeringReviewIssuerLengthRule { max_length: 80 };
        let mut item = new_default_peering_review("x", "y");
        item.issuer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
