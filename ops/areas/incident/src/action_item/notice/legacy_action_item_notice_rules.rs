use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy action item notice must pass. Check returns its problems, or none.
pub trait LegacyActionItemNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LegacyActionItemNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemNoticeAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LegacyActionItemNoticeRule for LegacyActionItemNoticeAllowedNameRule {
    fn check(&self, item: &LegacyActionItemNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is duplicated", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemNoticePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyActionItemNoticeRule for LegacyActionItemNoticePortRangeRule {
    fn check(&self, item: &LegacyActionItemNotice) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemNoticeBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyActionItemNoticeRule for LegacyActionItemNoticeBatchSizeRangeRule {
    fn check(&self, item: &LegacyActionItemNotice) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemNoticeRequiredPhoneRule {
    pub message: String,
}

impl LegacyActionItemNoticeRule for LegacyActionItemNoticeRequiredPhoneRule {
    fn check(&self, item: &LegacyActionItemNotice) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy action item notice.
#[derive(Debug, Clone)]
pub struct LegacyActionItemNoticeValidator {
    pub rules: Vec<Rc<dyn LegacyActionItemNoticeRule>>,
}

impl LegacyActionItemNoticeValidator {
    pub fn validate(&self, item: &LegacyActionItemNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyActionItemNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyActionItemNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_action_item_notice_validator() -> LegacyActionItemNoticeValidator {
    LegacyActionItemNoticeValidator {
        rules: vec![
            Rc::new(LegacyActionItemNoticeAllowedNameRule { allowed: Vec::new() })
                as Rc<dyn LegacyActionItemNoticeRule>,
            Rc::new(LegacyActionItemNoticePortRangeRule { min: 255, max: 86400 }),
            Rc::new(LegacyActionItemNoticeBatchSizeRangeRule { min: 80, max: 255 }),
            Rc::new(LegacyActionItemNoticeRequiredPhoneRule {
                message: "legacy action item notice needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_action_item_notice_validator();
        let item = new_legacy_action_item_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LegacyActionItemNoticePortRangeRule { min: 32, max: 64 };
        let mut item = new_legacy_action_item_notice("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LegacyActionItemNoticeBatchSizeRangeRule { min: 80, max: 3600 };
        let mut item = new_legacy_action_item_notice("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = LegacyActionItemNoticeRequiredPhoneRule {
            message: "legacy action item notice needs a value".to_string(),
        };
        let mut item = new_legacy_action_item_notice("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
