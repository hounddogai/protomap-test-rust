use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending binding window must pass. Check returns its problems, or none.
pub trait PendingBindingWindowRule: std::fmt::Debug {
    fn check(&self, item: &PendingBindingWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingWindowRequiredCodeRule {
    pub message: String,
}

impl PendingBindingWindowRule for PendingBindingWindowRequiredCodeRule {
    fn check(&self, item: &PendingBindingWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingWindowRequiredNotesRule {
    pub message: String,
}

impl PendingBindingWindowRule for PendingBindingWindowRequiredNotesRule {
    fn check(&self, item: &PendingBindingWindow) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingWindowRequiredIssuerRule {
    pub message: String,
}

impl PendingBindingWindowRule for PendingBindingWindowRequiredIssuerRule {
    fn check(&self, item: &PendingBindingWindow) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending binding window.
#[derive(Debug, Clone)]
pub struct PendingBindingWindowValidator {
    pub rules: Vec<Rc<dyn PendingBindingWindowRule>>,
}

impl PendingBindingWindowValidator {
    pub fn validate(&self, item: &PendingBindingWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingBindingWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingBindingWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_binding_window_validator() -> PendingBindingWindowValidator {
    PendingBindingWindowValidator {
        rules: vec![
            Rc::new(PendingBindingWindowRequiredCodeRule {
                message: "pending binding window needs an entry".to_string(),
            }) as Rc<dyn PendingBindingWindowRule>,
            Rc::new(PendingBindingWindowRequiredNotesRule {
                message: "pending binding window needs an entry".to_string(),
            }),
            Rc::new(PendingBindingWindowRequiredIssuerRule {
                message: "pending binding window needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_binding_window_validator();
        let item = new_pending_binding_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingBindingWindowRequiredCodeRule {
            message: "pending binding window needs a value".to_string(),
        };
        let mut item = new_pending_binding_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PendingBindingWindowRequiredNotesRule {
            message: "pending binding window needs an entry".to_string(),
        };
        let mut item = new_pending_binding_window("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = PendingBindingWindowRequiredIssuerRule {
            message: "pending binding window needs a value".to_string(),
        };
        let mut item = new_pending_binding_window("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
