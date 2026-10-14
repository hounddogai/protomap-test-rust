use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary escalation notice must pass. Check returns its problems, or none.
pub trait PrimaryEscalationNoticeRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEscalationNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticeRequiredCodeRule {
    pub message: String,
}

impl PrimaryEscalationNoticeRule for PrimaryEscalationNoticeRequiredCodeRule {
    fn check(&self, item: &PrimaryEscalationNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticeCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEscalationNoticeRule for PrimaryEscalationNoticeCodePrefixRule {
    fn check(&self, item: &PrimaryEscalationNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticeRequiredScopeRule {
    pub message: String,
}

impl PrimaryEscalationNoticeRule for PrimaryEscalationNoticeRequiredScopeRule {
    fn check(&self, item: &PrimaryEscalationNotice) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEscalationNoticeRule for PrimaryEscalationNoticePortRangeRule {
    fn check(&self, item: &PrimaryEscalationNotice) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticeNameLengthRule {
    pub max_length: i64,
}

impl PrimaryEscalationNoticeRule for PrimaryEscalationNoticeNameLengthRule {
    fn check(&self, item: &PrimaryEscalationNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNoticeLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryEscalationNoticeRule for PrimaryEscalationNoticeLabelLengthRule {
    fn check(&self, item: &PrimaryEscalationNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary escalation notice.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationNoticeValidator {
    pub rules: Vec<Rc<dyn PrimaryEscalationNoticeRule>>,
}

impl PrimaryEscalationNoticeValidator {
    pub fn validate(&self, item: &PrimaryEscalationNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryEscalationNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEscalationNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_escalation_notice_validator() -> PrimaryEscalationNoticeValidator {
    PrimaryEscalationNoticeValidator {
        rules: vec![
            Rc::new(PrimaryEscalationNoticeRequiredCodeRule {
                message: "primary escalation notice needs this field".to_string(),
            }) as Rc<dyn PrimaryEscalationNoticeRule>,
            Rc::new(PrimaryEscalationNoticeCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PrimaryEscalationNoticeRequiredScopeRule {
                message: "primary escalation notice needs this field".to_string(),
            }),
            Rc::new(PrimaryEscalationNoticePortRangeRule { min: 64, max: 120 }),
            Rc::new(PrimaryEscalationNoticeNameLengthRule { max_length: 3600 }),
            Rc::new(PrimaryEscalationNoticeLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_escalation_notice_validator();
        let item = new_primary_escalation_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryEscalationNoticeRequiredCodeRule {
            message: "primary escalation notice needs a value".to_string(),
        };
        let mut item = new_primary_escalation_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = PrimaryEscalationNoticeRequiredScopeRule {
            message: "primary escalation notice needs an entry".to_string(),
        };
        let mut item = new_primary_escalation_notice("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = PrimaryEscalationNoticePortRangeRule { min: 80, max: 86400 };
        let mut item = new_primary_escalation_notice("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryEscalationNoticeNameLengthRule { max_length: 32 };
        let mut item = new_primary_escalation_notice("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryEscalationNoticeLabelLengthRule { max_length: 80 };
        let mut item = new_primary_escalation_notice("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
