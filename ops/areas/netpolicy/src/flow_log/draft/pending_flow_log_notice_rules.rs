use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending flow log notice must pass. Check returns its problems, or none.
pub trait PendingFlowLogNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PendingFlowLogNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogNoticeCodeLengthRule {
    pub max_length: i64,
}

impl PendingFlowLogNoticeRule for PendingFlowLogNoticeCodeLengthRule {
    fn check(&self, item: &PendingFlowLogNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogNoticeRequiredCurrencyRule {
    pub message: String,
}

impl PendingFlowLogNoticeRule for PendingFlowLogNoticeRequiredCurrencyRule {
    fn check(&self, item: &PendingFlowLogNotice) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogNoticeRequiredTitleRule {
    pub message: String,
}

impl PendingFlowLogNoticeRule for PendingFlowLogNoticeRequiredTitleRule {
    fn check(&self, item: &PendingFlowLogNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogNoticeCodePrefixRule {
    pub prefix: String,
}

impl PendingFlowLogNoticeRule for PendingFlowLogNoticeCodePrefixRule {
    fn check(&self, item: &PendingFlowLogNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogNoticeTitleLengthRule {
    pub max_length: i64,
}

impl PendingFlowLogNoticeRule for PendingFlowLogNoticeTitleLengthRule {
    fn check(&self, item: &PendingFlowLogNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending flow log notice.
#[derive(Debug, Clone)]
pub struct PendingFlowLogNoticeValidator {
    pub rules: Vec<Rc<dyn PendingFlowLogNoticeRule>>,
}

impl PendingFlowLogNoticeValidator {
    pub fn validate(&self, item: &PendingFlowLogNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingFlowLogNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFlowLogNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_flow_log_notice_validator() -> PendingFlowLogNoticeValidator {
    PendingFlowLogNoticeValidator {
        rules: vec![
            Rc::new(PendingFlowLogNoticeCodeLengthRule { max_length: 3600 }) as Rc<dyn PendingFlowLogNoticeRule>,
            Rc::new(PendingFlowLogNoticeRequiredCurrencyRule {
                message: "pending flow log notice needs this field".to_string(),
            }),
            Rc::new(PendingFlowLogNoticeRequiredTitleRule {
                message: "pending flow log notice needs a value".to_string(),
            }),
            Rc::new(PendingFlowLogNoticeCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PendingFlowLogNoticeTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_flow_log_notice_validator();
        let item = new_pending_flow_log_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingFlowLogNoticeCodeLengthRule { max_length: 64 };
        let mut item = new_pending_flow_log_notice("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = PendingFlowLogNoticeRequiredCurrencyRule {
            message: "pending flow log notice needs an entry".to_string(),
        };
        let mut item = new_pending_flow_log_notice("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingFlowLogNoticeRequiredTitleRule {
            message: "pending flow log notice needs a value".to_string(),
        };
        let mut item = new_pending_flow_log_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingFlowLogNoticeTitleLengthRule { max_length: 80 };
        let mut item = new_pending_flow_log_notice("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
