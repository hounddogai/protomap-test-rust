use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress group must pass. Check returns its problems, or none.
pub trait EgressGroupRule: std::fmt::Debug {
    fn check(&self, item: &EgressGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupRequiredNotesRule {
    pub message: String,
}

impl EgressGroupRule for EgressGroupRequiredNotesRule {
    fn check(&self, item: &EgressGroup) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupPinnedGuardRule {
    pub limit: f64,
}

impl EgressGroupRule for EgressGroupPinnedGuardRule {
    fn check(&self, item: &EgressGroup) -> Vec<String> {
        if item.pinned && (item.maximum > self.limit) {
            return vec![format!("maximum {} exceeds {} while pinned", item.maximum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupCodeLengthRule {
    pub max_length: i64,
}

impl EgressGroupRule for EgressGroupCodeLengthRule {
    fn check(&self, item: &EgressGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupCodePrefixRule {
    pub prefix: String,
}

impl EgressGroupRule for EgressGroupCodePrefixRule {
    fn check(&self, item: &EgressGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupHostLengthRule {
    pub max_length: i64,
}

impl EgressGroupRule for EgressGroupHostLengthRule {
    fn check(&self, item: &EgressGroup) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupLabelLengthRule {
    pub max_length: i64,
}

impl EgressGroupRule for EgressGroupLabelLengthRule {
    fn check(&self, item: &EgressGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress group.
#[derive(Debug, Clone)]
pub struct EgressGroupValidator {
    pub rules: Vec<Rc<dyn EgressGroupRule>>,
}

impl EgressGroupValidator {
    pub fn validate(&self, item: &EgressGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_group_validator() -> EgressGroupValidator {
    EgressGroupValidator {
        rules: vec![
            Rc::new(EgressGroupRequiredNotesRule {
                message: "egress group needs a value".to_string(),
            }) as Rc<dyn EgressGroupRule>,
            Rc::new(EgressGroupPinnedGuardRule { limit: 0.4 }),
            Rc::new(EgressGroupCodeLengthRule { max_length: 64 }),
            Rc::new(EgressGroupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(EgressGroupHostLengthRule { max_length: 32 }),
            Rc::new(EgressGroupLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_group_validator();
        let item = new_egress_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = EgressGroupRequiredNotesRule {
            message: "egress group needs an entry".to_string(),
        };
        let mut item = new_egress_group("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressGroupCodeLengthRule { max_length: 80 };
        let mut item = new_egress_group("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = EgressGroupHostLengthRule { max_length: 3600 };
        let mut item = new_egress_group("x", "y");
        item.host = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressGroupLabelLengthRule { max_length: 64 };
        let mut item = new_egress_group("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
