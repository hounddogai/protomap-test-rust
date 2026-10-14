use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default zone review must pass. Check returns its problems, or none.
pub trait DefaultZoneReviewRule: std::fmt::Debug {
    fn check(&self, item: &DefaultZoneReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultZoneReviewRule for DefaultZoneReviewAllowedCodeRule {
    fn check(&self, item: &DefaultZoneReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewCodePrefixRule {
    pub prefix: String,
}

impl DefaultZoneReviewRule for DefaultZoneReviewCodePrefixRule {
    fn check(&self, item: &DefaultZoneReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewNameLengthRule {
    pub max_length: i64,
}

impl DefaultZoneReviewRule for DefaultZoneReviewNameLengthRule {
    fn check(&self, item: &DefaultZoneReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultZoneReviewRule for DefaultZoneReviewCostRangeRule {
    fn check(&self, item: &DefaultZoneReview) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultZoneReviewActiveGuardRule {
    pub limit: f64,
}

impl DefaultZoneReviewRule for DefaultZoneReviewActiveGuardRule {
    fn check(&self, item: &DefaultZoneReview) -> Vec<String> {
        if item.active && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while active", item.cost, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default zone review.
#[derive(Debug, Clone)]
pub struct DefaultZoneReviewValidator {
    pub rules: Vec<Rc<dyn DefaultZoneReviewRule>>,
}

impl DefaultZoneReviewValidator {
    pub fn validate(&self, item: &DefaultZoneReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultZoneReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultZoneReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_zone_review_validator() -> DefaultZoneReviewValidator {
    DefaultZoneReviewValidator {
        rules: vec![
            Rc::new(DefaultZoneReviewAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn DefaultZoneReviewRule>,
            Rc::new(DefaultZoneReviewCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DefaultZoneReviewNameLengthRule { max_length: 80 }),
            Rc::new(DefaultZoneReviewCostRangeRule { min: 3.0, max: 3174.5 }),
            Rc::new(DefaultZoneReviewActiveGuardRule { limit: 5.2 }),
        ],
    }
}
