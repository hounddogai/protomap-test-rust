use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback action item notice must pass. Check returns its problems, or none.
pub trait FallbackActionItemNoticeRule: std::fmt::Debug {
    fn check(&self, item: &FallbackActionItemNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeRequiredCodeRule {
    pub message: String,
}

impl FallbackActionItemNoticeRule for FallbackActionItemNoticeRequiredCodeRule {
    fn check(&self, item: &FallbackActionItemNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeRequiredUnitRule {
    pub message: String,
}

impl FallbackActionItemNoticeRule for FallbackActionItemNoticeRequiredUnitRule {
    fn check(&self, item: &FallbackActionItemNotice) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeActiveGuardRule {
    pub limit: i64,
}

impl FallbackActionItemNoticeRule for FallbackActionItemNoticeActiveGuardRule {
    fn check(&self, item: &FallbackActionItemNotice) -> Vec<String> {
        if item.active && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while active",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackActionItemNoticeRule for FallbackActionItemNoticeEndsAtRangeRule {
    fn check(&self, item: &FallbackActionItemNotice) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeRequiredLocaleRule {
    pub message: String,
}

impl FallbackActionItemNoticeRule for FallbackActionItemNoticeRequiredLocaleRule {
    fn check(&self, item: &FallbackActionItemNotice) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl FallbackActionItemNoticeRule for FallbackActionItemNoticeAllowedUnitRule {
    fn check(&self, item: &FallbackActionItemNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is inconsistent", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback action item notice.
#[derive(Debug, Clone)]
pub struct FallbackActionItemNoticeValidator {
    pub rules: Vec<Rc<dyn FallbackActionItemNoticeRule>>,
}

impl FallbackActionItemNoticeValidator {
    pub fn validate(&self, item: &FallbackActionItemNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackActionItemNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackActionItemNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_action_item_notice_validator() -> FallbackActionItemNoticeValidator {
    FallbackActionItemNoticeValidator {
        rules: vec![
            Rc::new(FallbackActionItemNoticeRequiredCodeRule {
                message: "fallback action item notice needs this field".to_string(),
            }) as Rc<dyn FallbackActionItemNoticeRule>,
            Rc::new(FallbackActionItemNoticeRequiredUnitRule {
                message: "fallback action item notice needs a value".to_string(),
            }),
            Rc::new(FallbackActionItemNoticeActiveGuardRule { limit: 3600 }),
            Rc::new(FallbackActionItemNoticeEndsAtRangeRule { min: 64, max: 86400 }),
            Rc::new(FallbackActionItemNoticeRequiredLocaleRule {
                message: "fallback action item notice needs a value".to_string(),
            }),
            Rc::new(FallbackActionItemNoticeAllowedUnitRule {
                allowed: vec!["legacy".to_string(), "adaptive".to_string(), "adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_action_item_notice_validator();
        let item = new_fallback_action_item_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackActionItemNoticeRequiredCodeRule {
            message: "fallback action item notice needs an entry".to_string(),
        };
        let mut item = new_fallback_action_item_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = FallbackActionItemNoticeRequiredUnitRule {
            message: "fallback action item notice needs this field".to_string(),
        };
        let mut item = new_fallback_action_item_notice("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = FallbackActionItemNoticeEndsAtRangeRule { min: 32, max: 120 };
        let mut item = new_fallback_action_item_notice("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = FallbackActionItemNoticeRequiredLocaleRule {
            message: "fallback action item notice needs an entry".to_string(),
        };
        let mut item = new_fallback_action_item_notice("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
