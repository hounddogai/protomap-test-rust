use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive rollback notice must pass. Check returns its problems, or none.
pub trait AdaptiveRollbackNoticeRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRollbackNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNoticeDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRollbackNoticeRule for AdaptiveRollbackNoticeDiscountRangeRule {
    fn check(&self, item: &AdaptiveRollbackNotice) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNoticeRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRollbackNoticeRule for AdaptiveRollbackNoticeRequiredCodeRule {
    fn check(&self, item: &AdaptiveRollbackNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNoticeCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRollbackNoticeRule for AdaptiveRollbackNoticeCodeLengthRule {
    fn check(&self, item: &AdaptiveRollbackNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNoticeSourceLengthRule {
    pub max_length: i64,
}

impl AdaptiveRollbackNoticeRule for AdaptiveRollbackNoticeSourceLengthRule {
    fn check(&self, item: &AdaptiveRollbackNotice) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNoticeLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveRollbackNoticeRule for AdaptiveRollbackNoticeLabelLengthRule {
    fn check(&self, item: &AdaptiveRollbackNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackNoticeAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRollbackNoticeRule for AdaptiveRollbackNoticeAllowedSourceRule {
    fn check(&self, item: &AdaptiveRollbackNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is not allowed", item.source)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive rollback notice.
#[derive(Debug, Clone)]
pub struct AdaptiveRollbackNoticeValidator {
    pub rules: Vec<Rc<dyn AdaptiveRollbackNoticeRule>>,
}

impl AdaptiveRollbackNoticeValidator {
    pub fn validate(&self, item: &AdaptiveRollbackNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveRollbackNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRollbackNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rollback_notice_validator() -> AdaptiveRollbackNoticeValidator {
    AdaptiveRollbackNoticeValidator {
        rules: vec![
            Rc::new(AdaptiveRollbackNoticeDiscountRangeRule { min: 1.1, max: 3797.0 })
                as Rc<dyn AdaptiveRollbackNoticeRule>,
            Rc::new(AdaptiveRollbackNoticeRequiredCodeRule {
                message: "adaptive rollback notice needs a value".to_string(),
            }),
            Rc::new(AdaptiveRollbackNoticeCodeLengthRule { max_length: 86400 }),
            Rc::new(AdaptiveRollbackNoticeSourceLengthRule { max_length: 16 }),
            Rc::new(AdaptiveRollbackNoticeLabelLengthRule { max_length: 32 }),
            Rc::new(AdaptiveRollbackNoticeAllowedSourceRule {
                allowed: vec!["fallback".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_rollback_notice_validator();
        let item = new_adaptive_rollback_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = AdaptiveRollbackNoticeDiscountRangeRule { min: 8.4, max: 1746.0 };
        let mut item = new_adaptive_rollback_notice("x", "y");
        item.discount = 1747.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveRollbackNoticeRequiredCodeRule {
            message: "adaptive rollback notice needs an entry".to_string(),
        };
        let mut item = new_adaptive_rollback_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveRollbackNoticeCodeLengthRule { max_length: 32 };
        let mut item = new_adaptive_rollback_notice("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = AdaptiveRollbackNoticeSourceLengthRule { max_length: 86400 };
        let mut item = new_adaptive_rollback_notice("x", "y");
        item.source = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveRollbackNoticeLabelLengthRule { max_length: 16 };
        let mut item = new_adaptive_rollback_notice("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
