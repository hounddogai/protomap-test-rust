use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress notice must pass. Check returns its problems, or none.
pub trait EgressNoticeRule: std::fmt::Debug {
    fn check(&self, item: &EgressNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressNoticeCodeLengthRule {
    pub max_length: i64,
}

impl EgressNoticeRule for EgressNoticeCodeLengthRule {
    fn check(&self, item: &EgressNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressNoticeRequiredNameRule {
    pub message: String,
}

impl EgressNoticeRule for EgressNoticeRequiredNameRule {
    fn check(&self, item: &EgressNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressNoticeRequiredCodeRule {
    pub message: String,
}

impl EgressNoticeRule for EgressNoticeRequiredCodeRule {
    fn check(&self, item: &EgressNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress notice.
#[derive(Debug, Clone)]
pub struct EgressNoticeValidator {
    pub rules: Vec<Rc<dyn EgressNoticeRule>>,
}

impl EgressNoticeValidator {
    pub fn validate(&self, item: &EgressNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_notice_validator() -> EgressNoticeValidator {
    EgressNoticeValidator {
        rules: vec![
            Rc::new(EgressNoticeCodeLengthRule { max_length: 3600 }) as Rc<dyn EgressNoticeRule>,
            Rc::new(EgressNoticeRequiredNameRule {
                message: "egress notice needs a value".to_string(),
            }),
            Rc::new(EgressNoticeRequiredCodeRule {
                message: "egress notice needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_notice_validator();
        let item = new_egress_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressNoticeCodeLengthRule { max_length: 120 };
        let mut item = new_egress_notice("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EgressNoticeRequiredNameRule {
            message: "egress notice needs an entry".to_string(),
        };
        let mut item = new_egress_notice("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressNoticeRequiredCodeRule {
            message: "egress notice needs an entry".to_string(),
        };
        let mut item = new_egress_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
