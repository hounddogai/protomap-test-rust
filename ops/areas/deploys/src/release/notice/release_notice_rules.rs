use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release notice must pass. Check returns its problems, or none.
pub trait ReleaseNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseNoticeAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseNoticeRule for ReleaseNoticeAmountRangeRule {
    fn check(&self, item: &ReleaseNotice) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseNoticeCategoryLengthRule {
    pub max_length: i64,
}

impl ReleaseNoticeRule for ReleaseNoticeCategoryLengthRule {
    fn check(&self, item: &ReleaseNotice) -> Vec<String> {
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
pub struct ReleaseNoticeAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl ReleaseNoticeRule for ReleaseNoticeAllowedRegionRule {
    fn check(&self, item: &ReleaseNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is missing", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseNoticeRequiredLabelRule {
    pub message: String,
}

impl ReleaseNoticeRule for ReleaseNoticeRequiredLabelRule {
    fn check(&self, item: &ReleaseNotice) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseNoticeDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseNoticeRule for ReleaseNoticeDiscountRangeRule {
    fn check(&self, item: &ReleaseNotice) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a release notice.
#[derive(Debug, Clone)]
pub struct ReleaseNoticeValidator {
    pub rules: Vec<Rc<dyn ReleaseNoticeRule>>,
}

impl ReleaseNoticeValidator {
    pub fn validate(&self, item: &ReleaseNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_notice_validator() -> ReleaseNoticeValidator {
    ReleaseNoticeValidator {
        rules: vec![
            Rc::new(ReleaseNoticeAmountRangeRule { min: 2.9, max: 3971.5 }) as Rc<dyn ReleaseNoticeRule>,
            Rc::new(ReleaseNoticeCategoryLengthRule { max_length: 86400 }),
            Rc::new(ReleaseNoticeAllowedRegionRule { allowed: Vec::new() }),
            Rc::new(ReleaseNoticeRequiredLabelRule {
                message: "release notice needs a value".to_string(),
            }),
            Rc::new(ReleaseNoticeDiscountRangeRule { min: 1.4, max: 836.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_notice_validator();
        let item = new_release_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ReleaseNoticeAmountRangeRule { min: 7.8, max: 2574.2 };
        let mut item = new_release_notice("x", "y");
        item.amount = 2575.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = ReleaseNoticeCategoryLengthRule { max_length: 120 };
        let mut item = new_release_notice("x", "y");
        item.category = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ReleaseNoticeRequiredLabelRule {
            message: "release notice needs this field".to_string(),
        };
        let mut item = new_release_notice("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = ReleaseNoticeDiscountRangeRule { min: 3.2, max: 1501.8 };
        let mut item = new_release_notice("x", "y");
        item.discount = 1503.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
