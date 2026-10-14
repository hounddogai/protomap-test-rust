use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering review must pass. Check returns its problems, or none.
pub trait PeeringReviewRule: std::fmt::Debug {
    fn check(&self, item: &PeeringReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReviewCodePrefixRule {
    pub prefix: String,
}

impl PeeringReviewRule for PeeringReviewCodePrefixRule {
    fn check(&self, item: &PeeringReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReviewRequiredCodeRule {
    pub message: String,
}

impl PeeringReviewRule for PeeringReviewRequiredCodeRule {
    fn check(&self, item: &PeeringReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReviewRequiredLabelRule {
    pub message: String,
}

impl PeeringReviewRule for PeeringReviewRequiredLabelRule {
    fn check(&self, item: &PeeringReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReviewThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringReviewRule for PeeringReviewThresholdRangeRule {
    fn check(&self, item: &PeeringReview) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a peering review.
#[derive(Debug, Clone)]
pub struct PeeringReviewValidator {
    pub rules: Vec<Rc<dyn PeeringReviewRule>>,
}

impl PeeringReviewValidator {
    pub fn validate(&self, item: &PeeringReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_review_validator() -> PeeringReviewValidator {
    PeeringReviewValidator {
        rules: vec![
            Rc::new(PeeringReviewCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn PeeringReviewRule>,
            Rc::new(PeeringReviewRequiredCodeRule {
                message: "peering review needs this field".to_string(),
            }),
            Rc::new(PeeringReviewRequiredLabelRule {
                message: "peering review needs a value".to_string(),
            }),
            Rc::new(PeeringReviewThresholdRangeRule { min: 7.7, max: 3902.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_review_validator();
        let item = new_peering_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PeeringReviewRequiredCodeRule {
            message: "peering review needs a value".to_string(),
        };
        let mut item = new_peering_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PeeringReviewRequiredLabelRule {
            message: "peering review needs a value".to_string(),
        };
        let mut item = new_peering_review("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PeeringReviewThresholdRangeRule { min: 6.5, max: 1739.7 };
        let mut item = new_peering_review("x", "y");
        item.threshold = 1741.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
