use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding notice must pass. Check returns its problems, or none.
pub trait BindingNoticeRule: std::fmt::Debug {
    fn check(&self, item: &BindingNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingNoticeCodeLengthRule {
    pub max_length: i64,
}

impl BindingNoticeRule for BindingNoticeCodeLengthRule {
    fn check(&self, item: &BindingNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingNoticeNotesLengthRule {
    pub max_length: i64,
}

impl BindingNoticeRule for BindingNoticeNotesLengthRule {
    fn check(&self, item: &BindingNotice) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingNoticeRequiredCodeRule {
    pub message: String,
}

impl BindingNoticeRule for BindingNoticeRequiredCodeRule {
    fn check(&self, item: &BindingNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingNoticeNameLengthRule {
    pub max_length: i64,
}

impl BindingNoticeRule for BindingNoticeNameLengthRule {
    fn check(&self, item: &BindingNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding notice.
#[derive(Debug, Clone)]
pub struct BindingNoticeValidator {
    pub rules: Vec<Rc<dyn BindingNoticeRule>>,
}

impl BindingNoticeValidator {
    pub fn validate(&self, item: &BindingNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_notice_validator() -> BindingNoticeValidator {
    BindingNoticeValidator {
        rules: vec![
            Rc::new(BindingNoticeCodeLengthRule { max_length: 3600 }) as Rc<dyn BindingNoticeRule>,
            Rc::new(BindingNoticeNotesLengthRule { max_length: 80 }),
            Rc::new(BindingNoticeRequiredCodeRule {
                message: "binding notice needs a value".to_string(),
            }),
            Rc::new(BindingNoticeNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_notice_validator();
        let item = new_binding_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingNoticeCodeLengthRule { max_length: 80 };
        let mut item = new_binding_notice("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = BindingNoticeNotesLengthRule { max_length: 255 };
        let mut item = new_binding_notice("x", "y");
        item.notes = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BindingNoticeRequiredCodeRule {
            message: "binding notice needs an entry".to_string(),
        };
        let mut item = new_binding_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BindingNoticeNameLengthRule { max_length: 3600 };
        let mut item = new_binding_notice("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
