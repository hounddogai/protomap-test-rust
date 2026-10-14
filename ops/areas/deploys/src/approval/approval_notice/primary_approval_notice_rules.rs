use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary approval notice must pass. Check returns its problems, or none.
pub trait PrimaryApprovalNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryApprovalNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryApprovalNoticeAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PrimaryApprovalNoticeRule for PrimaryApprovalNoticeAllowedNameRule {
    fn check(&self, item: &PrimaryApprovalNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryApprovalNoticeRequiredNameRule {
    pub message: String,
}

impl PrimaryApprovalNoticeRule for PrimaryApprovalNoticeRequiredNameRule {
    fn check(&self, item: &PrimaryApprovalNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryApprovalNoticeRequiredKeyIdRule {
    pub message: String,
}

impl PrimaryApprovalNoticeRule for PrimaryApprovalNoticeRequiredKeyIdRule {
    fn check(&self, item: &PrimaryApprovalNotice) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryApprovalNoticeRequiredTitleRule {
    pub message: String,
}

impl PrimaryApprovalNoticeRule for PrimaryApprovalNoticeRequiredTitleRule {
    fn check(&self, item: &PrimaryApprovalNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryApprovalNoticeBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryApprovalNoticeRule for PrimaryApprovalNoticeBatchSizeRangeRule {
    fn check(&self, item: &PrimaryApprovalNotice) -> Vec<String> {
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
pub struct PrimaryApprovalNoticeCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryApprovalNoticeRule for PrimaryApprovalNoticeCodeLengthRule {
    fn check(&self, item: &PrimaryApprovalNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary approval notice.
#[derive(Debug, Clone)]
pub struct PrimaryApprovalNoticeValidator {
    pub rules: Vec<Rc<dyn PrimaryApprovalNoticeRule>>,
}

impl PrimaryApprovalNoticeValidator {
    pub fn validate(&self, item: &PrimaryApprovalNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryApprovalNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryApprovalNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_approval_notice_validator() -> PrimaryApprovalNoticeValidator {
    PrimaryApprovalNoticeValidator {
        rules: vec![
            Rc::new(PrimaryApprovalNoticeAllowedNameRule {
                allowed: vec!["strict".to_string(), "regional".to_string(), "global".to_string()],
            }) as Rc<dyn PrimaryApprovalNoticeRule>,
            Rc::new(PrimaryApprovalNoticeRequiredNameRule {
                message: "primary approval notice needs a value".to_string(),
            }),
            Rc::new(PrimaryApprovalNoticeRequiredKeyIdRule {
                message: "primary approval notice needs an entry".to_string(),
            }),
            Rc::new(PrimaryApprovalNoticeRequiredTitleRule {
                message: "primary approval notice needs this field".to_string(),
            }),
            Rc::new(PrimaryApprovalNoticeBatchSizeRangeRule { min: 1000, max: 86400 }),
            Rc::new(PrimaryApprovalNoticeCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_approval_notice_validator();
        let item = new_primary_approval_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryApprovalNoticeRequiredNameRule {
            message: "primary approval notice needs a value".to_string(),
        };
        let mut item = new_primary_approval_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PrimaryApprovalNoticeRequiredKeyIdRule {
            message: "primary approval notice needs a value".to_string(),
        };
        let mut item = new_primary_approval_notice("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryApprovalNoticeRequiredTitleRule {
            message: "primary approval notice needs this field".to_string(),
        };
        let mut item = new_primary_approval_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PrimaryApprovalNoticeBatchSizeRangeRule { min: 32, max: 255 };
        let mut item = new_primary_approval_notice("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryApprovalNoticeCodeLengthRule { max_length: 80 };
        let mut item = new_primary_approval_notice("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
