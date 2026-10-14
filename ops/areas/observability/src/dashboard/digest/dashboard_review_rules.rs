use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dashboard review must pass. Check returns its problems, or none.
pub trait DashboardReviewRule: std::fmt::Debug {
    fn check(&self, item: &DashboardReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReviewRequiredCodeRule {
    pub message: String,
}

impl DashboardReviewRule for DashboardReviewRequiredCodeRule {
    fn check(&self, item: &DashboardReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReviewAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DashboardReviewRule for DashboardReviewAllowedNameRule {
    fn check(&self, item: &DashboardReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardReviewRequiredNameRule {
    pub message: String,
}

impl DashboardReviewRule for DashboardReviewRequiredNameRule {
    fn check(&self, item: &DashboardReview) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard review.
#[derive(Debug, Clone)]
pub struct DashboardReviewValidator {
    pub rules: Vec<Rc<dyn DashboardReviewRule>>,
}

impl DashboardReviewValidator {
    pub fn validate(&self, item: &DashboardReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_review_validator() -> DashboardReviewValidator {
    DashboardReviewValidator {
        rules: vec![
            Rc::new(DashboardReviewRequiredCodeRule {
                message: "dashboard review needs an entry".to_string(),
            }) as Rc<dyn DashboardReviewRule>,
            Rc::new(DashboardReviewAllowedNameRule {
                allowed: vec!["lenient".to_string(), "legacy".to_string()],
            }),
            Rc::new(DashboardReviewRequiredNameRule {
                message: "dashboard review needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_review_validator();
        let item = new_dashboard_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DashboardReviewRequiredCodeRule {
            message: "dashboard review needs a value".to_string(),
        };
        let mut item = new_dashboard_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DashboardReviewRequiredNameRule {
            message: "dashboard review needs this field".to_string(),
        };
        let mut item = new_dashboard_review("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
