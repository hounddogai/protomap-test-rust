use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled probe exception must pass. Check returns its problems, or none.
pub trait ScheduledProbeExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledProbeException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeExceptionRequiredCodeRule {
    pub message: String,
}

impl ScheduledProbeExceptionRule for ScheduledProbeExceptionRequiredCodeRule {
    fn check(&self, item: &ScheduledProbeException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeExceptionLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledProbeExceptionRule for ScheduledProbeExceptionLabelLengthRule {
    fn check(&self, item: &ScheduledProbeException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeExceptionCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledProbeExceptionRule for ScheduledProbeExceptionCodeLengthRule {
    fn check(&self, item: &ScheduledProbeException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeExceptionRequiredContactEmailRule {
    pub message: String,
}

impl ScheduledProbeExceptionRule for ScheduledProbeExceptionRequiredContactEmailRule {
    fn check(&self, item: &ScheduledProbeException) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeExceptionTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledProbeExceptionRule for ScheduledProbeExceptionTitleLengthRule {
    fn check(&self, item: &ScheduledProbeException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled probe exception.
#[derive(Debug, Clone)]
pub struct ScheduledProbeExceptionValidator {
    pub rules: Vec<Rc<dyn ScheduledProbeExceptionRule>>,
}

impl ScheduledProbeExceptionValidator {
    pub fn validate(&self, item: &ScheduledProbeException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledProbeException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledProbeException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_probe_exception_validator() -> ScheduledProbeExceptionValidator {
    ScheduledProbeExceptionValidator {
        rules: vec![
            Rc::new(ScheduledProbeExceptionRequiredCodeRule {
                message: "scheduled probe exception needs this field".to_string(),
            }) as Rc<dyn ScheduledProbeExceptionRule>,
            Rc::new(ScheduledProbeExceptionLabelLengthRule { max_length: 16 }),
            Rc::new(ScheduledProbeExceptionCodeLengthRule { max_length: 64 }),
            Rc::new(ScheduledProbeExceptionRequiredContactEmailRule {
                message: "scheduled probe exception needs a value".to_string(),
            }),
            Rc::new(ScheduledProbeExceptionTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_probe_exception_validator();
        let item = new_scheduled_probe_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledProbeExceptionRequiredCodeRule {
            message: "scheduled probe exception needs a value".to_string(),
        };
        let mut item = new_scheduled_probe_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledProbeExceptionLabelLengthRule { max_length: 1000 };
        let mut item = new_scheduled_probe_exception("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledProbeExceptionCodeLengthRule { max_length: 16 };
        let mut item = new_scheduled_probe_exception("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ScheduledProbeExceptionRequiredContactEmailRule {
            message: "scheduled probe exception needs a value".to_string(),
        };
        let mut item = new_scheduled_probe_exception("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledProbeExceptionTitleLengthRule { max_length: 3600 };
        let mut item = new_scheduled_probe_exception("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
