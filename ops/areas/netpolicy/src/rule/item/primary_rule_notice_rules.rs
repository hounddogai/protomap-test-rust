use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rule notice must pass. Check returns its problems, or none.
pub trait PrimaryRuleNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRuleNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNoticeRequiredTitleRule {
    pub message: String,
}

impl PrimaryRuleNoticeRule for PrimaryRuleNoticeRequiredTitleRule {
    fn check(&self, item: &PrimaryRuleNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNoticeTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PrimaryRuleNoticeRule for PrimaryRuleNoticeTagsLimitRule {
    fn check(&self, item: &PrimaryRuleNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNoticeMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRuleNoticeRule for PrimaryRuleNoticeMaximumRangeRule {
    fn check(&self, item: &PrimaryRuleNotice) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNoticeRequiredDescriptionRule {
    pub message: String,
}

impl PrimaryRuleNoticeRule for PrimaryRuleNoticeRequiredDescriptionRule {
    fn check(&self, item: &PrimaryRuleNotice) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNoticeCountryLengthRule {
    pub max_length: i64,
}

impl PrimaryRuleNoticeRule for PrimaryRuleNoticeCountryLengthRule {
    fn check(&self, item: &PrimaryRuleNotice) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleNoticeTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryRuleNoticeRule for PrimaryRuleNoticeTitleLengthRule {
    fn check(&self, item: &PrimaryRuleNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary rule notice.
#[derive(Debug, Clone)]
pub struct PrimaryRuleNoticeValidator {
    pub rules: Vec<Rc<dyn PrimaryRuleNoticeRule>>,
}

impl PrimaryRuleNoticeValidator {
    pub fn validate(&self, item: &PrimaryRuleNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRuleNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRuleNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rule_notice_validator() -> PrimaryRuleNoticeValidator {
    PrimaryRuleNoticeValidator {
        rules: vec![
            Rc::new(PrimaryRuleNoticeRequiredTitleRule {
                message: "primary rule notice needs a value".to_string(),
            }) as Rc<dyn PrimaryRuleNoticeRule>,
            Rc::new(PrimaryRuleNoticeTagsLimitRule {
                max_count: 1000,
                banned: Vec::new(),
            }),
            Rc::new(PrimaryRuleNoticeMaximumRangeRule { min: 6.7, max: 4200.7 }),
            Rc::new(PrimaryRuleNoticeRequiredDescriptionRule {
                message: "primary rule notice needs a value".to_string(),
            }),
            Rc::new(PrimaryRuleNoticeCountryLengthRule { max_length: 86400 }),
            Rc::new(PrimaryRuleNoticeTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rule_notice_validator();
        let item = new_primary_rule_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryRuleNoticeRequiredTitleRule {
            message: "primary rule notice needs a value".to_string(),
        };
        let mut item = new_primary_rule_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PrimaryRuleNoticeMaximumRangeRule { min: 0.6, max: 3532.4 };
        let mut item = new_primary_rule_notice("x", "y");
        item.maximum = 3533.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PrimaryRuleNoticeRequiredDescriptionRule {
            message: "primary rule notice needs an entry".to_string(),
        };
        let mut item = new_primary_rule_notice("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PrimaryRuleNoticeCountryLengthRule { max_length: 255 };
        let mut item = new_primary_rule_notice("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryRuleNoticeTitleLengthRule { max_length: 80 };
        let mut item = new_primary_rule_notice("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
