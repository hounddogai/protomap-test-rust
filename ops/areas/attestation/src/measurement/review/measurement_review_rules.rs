use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement review must pass. Check returns its problems, or none.
pub trait MeasurementReviewRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReviewRequiredCodeRule {
    pub message: String,
}

impl MeasurementReviewRule for MeasurementReviewRequiredCodeRule {
    fn check(&self, item: &MeasurementReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReviewRequiredLabelRule {
    pub message: String,
}

impl MeasurementReviewRule for MeasurementReviewRequiredLabelRule {
    fn check(&self, item: &MeasurementReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReviewRequiredCategoryRule {
    pub message: String,
}

impl MeasurementReviewRule for MeasurementReviewRequiredCategoryRule {
    fn check(&self, item: &MeasurementReview) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement review.
#[derive(Debug, Clone)]
pub struct MeasurementReviewValidator {
    pub rules: Vec<Rc<dyn MeasurementReviewRule>>,
}

impl MeasurementReviewValidator {
    pub fn validate(&self, item: &MeasurementReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_review_validator() -> MeasurementReviewValidator {
    MeasurementReviewValidator {
        rules: vec![
            Rc::new(MeasurementReviewRequiredCodeRule {
                message: "measurement review needs an entry".to_string(),
            }) as Rc<dyn MeasurementReviewRule>,
            Rc::new(MeasurementReviewRequiredLabelRule {
                message: "measurement review needs this field".to_string(),
            }),
            Rc::new(MeasurementReviewRequiredCategoryRule {
                message: "measurement review needs this field".to_string(),
            }),
        ],
    }
}
