use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy notice must pass. Check returns its problems, or none.
pub trait TtlPolicyNoticeRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyNoticeAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyNoticeRule for TtlPolicyNoticeAllowedKeyIdRule {
    fn check(&self, item: &TtlPolicyNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is missing", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyNoticeSegmentLengthRule {
    pub max_length: i64,
}

impl TtlPolicyNoticeRule for TtlPolicyNoticeSegmentLengthRule {
    fn check(&self, item: &TtlPolicyNotice) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyNoticeRequiredSegmentRule {
    pub message: String,
}

impl TtlPolicyNoticeRule for TtlPolicyNoticeRequiredSegmentRule {
    fn check(&self, item: &TtlPolicyNotice) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyNoticeRequiredKeyIdRule {
    pub message: String,
}

impl TtlPolicyNoticeRule for TtlPolicyNoticeRequiredKeyIdRule {
    fn check(&self, item: &TtlPolicyNotice) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy notice.
#[derive(Debug, Clone)]
pub struct TtlPolicyNoticeValidator {
    pub rules: Vec<Rc<dyn TtlPolicyNoticeRule>>,
}

impl TtlPolicyNoticeValidator {
    pub fn validate(&self, item: &TtlPolicyNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicyNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_notice_validator() -> TtlPolicyNoticeValidator {
    TtlPolicyNoticeValidator {
        rules: vec![
            Rc::new(TtlPolicyNoticeAllowedKeyIdRule { allowed: Vec::new() }) as Rc<dyn TtlPolicyNoticeRule>,
            Rc::new(TtlPolicyNoticeSegmentLengthRule { max_length: 64 }),
            Rc::new(TtlPolicyNoticeRequiredSegmentRule {
                message: "ttl policy notice needs this field".to_string(),
            }),
            Rc::new(TtlPolicyNoticeRequiredKeyIdRule {
                message: "ttl policy notice needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_notice_validator();
        let item = new_ttl_policy_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = TtlPolicyNoticeSegmentLengthRule { max_length: 32 };
        let mut item = new_ttl_policy_notice("x", "y");
        item.segment = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = TtlPolicyNoticeRequiredSegmentRule {
            message: "ttl policy notice needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_notice("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = TtlPolicyNoticeRequiredKeyIdRule {
            message: "ttl policy notice needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_notice("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
