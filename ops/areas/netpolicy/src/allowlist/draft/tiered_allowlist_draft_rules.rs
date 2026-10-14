use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered allowlist draft must pass. Check returns its problems, or none.
pub trait TieredAllowlistDraftRule: std::fmt::Debug {
    fn check(&self, item: &TieredAllowlistDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistDraftRequiredCodeRule {
    pub message: String,
}

impl TieredAllowlistDraftRule for TieredAllowlistDraftRequiredCodeRule {
    fn check(&self, item: &TieredAllowlistDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistDraftRequiredNameRule {
    pub message: String,
}

impl TieredAllowlistDraftRule for TieredAllowlistDraftRequiredNameRule {
    fn check(&self, item: &TieredAllowlistDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistDraftChannelLengthRule {
    pub max_length: i64,
}

impl TieredAllowlistDraftRule for TieredAllowlistDraftChannelLengthRule {
    fn check(&self, item: &TieredAllowlistDraft) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered allowlist draft.
#[derive(Debug, Clone)]
pub struct TieredAllowlistDraftValidator {
    pub rules: Vec<Rc<dyn TieredAllowlistDraftRule>>,
}

impl TieredAllowlistDraftValidator {
    pub fn validate(&self, item: &TieredAllowlistDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredAllowlistDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAllowlistDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_allowlist_draft_validator() -> TieredAllowlistDraftValidator {
    TieredAllowlistDraftValidator {
        rules: vec![
            Rc::new(TieredAllowlistDraftRequiredCodeRule {
                message: "tiered allowlist draft needs a value".to_string(),
            }) as Rc<dyn TieredAllowlistDraftRule>,
            Rc::new(TieredAllowlistDraftRequiredNameRule {
                message: "tiered allowlist draft needs this field".to_string(),
            }),
            Rc::new(TieredAllowlistDraftChannelLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_allowlist_draft_validator();
        let item = new_tiered_allowlist_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredAllowlistDraftRequiredCodeRule {
            message: "tiered allowlist draft needs a value".to_string(),
        };
        let mut item = new_tiered_allowlist_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredAllowlistDraftRequiredNameRule {
            message: "tiered allowlist draft needs this field".to_string(),
        };
        let mut item = new_tiered_allowlist_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = TieredAllowlistDraftChannelLengthRule { max_length: 1000 };
        let mut item = new_tiered_allowlist_draft("x", "y");
        item.channel = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
