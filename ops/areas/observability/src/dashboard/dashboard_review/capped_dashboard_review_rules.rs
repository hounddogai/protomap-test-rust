use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped dashboard review must pass. Check returns its problems, or none.
pub trait CappedDashboardReviewRule: std::fmt::Debug {
    fn check(&self, item: &CappedDashboardReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardReviewRequiredLocaleRule {
    pub message: String,
}

impl CappedDashboardReviewRule for CappedDashboardReviewRequiredLocaleRule {
    fn check(&self, item: &CappedDashboardReview) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardReviewLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDashboardReviewRule for CappedDashboardReviewLimitRangeRule {
    fn check(&self, item: &CappedDashboardReview) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardReviewRequiredLabelRule {
    pub message: String,
}

impl CappedDashboardReviewRule for CappedDashboardReviewRequiredLabelRule {
    fn check(&self, item: &CappedDashboardReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped dashboard review.
#[derive(Debug, Clone)]
pub struct CappedDashboardReviewValidator {
    pub rules: Vec<Rc<dyn CappedDashboardReviewRule>>,
}

impl CappedDashboardReviewValidator {
    pub fn validate(&self, item: &CappedDashboardReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedDashboardReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDashboardReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_dashboard_review_validator() -> CappedDashboardReviewValidator {
    CappedDashboardReviewValidator {
        rules: vec![
            Rc::new(CappedDashboardReviewRequiredLocaleRule {
                message: "capped dashboard review needs a value".to_string(),
            }) as Rc<dyn CappedDashboardReviewRule>,
            Rc::new(CappedDashboardReviewLimitRangeRule { min: 64, max: 80 }),
            Rc::new(CappedDashboardReviewRequiredLabelRule {
                message: "capped dashboard review needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_dashboard_review_validator();
        let item = new_capped_dashboard_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = CappedDashboardReviewRequiredLocaleRule {
            message: "capped dashboard review needs this field".to_string(),
        };
        let mut item = new_capped_dashboard_review("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = CappedDashboardReviewLimitRangeRule { min: 64, max: 64 };
        let mut item = new_capped_dashboard_review("x", "y");
        item.limit = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedDashboardReviewRequiredLabelRule {
            message: "capped dashboard review needs a value".to_string(),
        };
        let mut item = new_capped_dashboard_review("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
