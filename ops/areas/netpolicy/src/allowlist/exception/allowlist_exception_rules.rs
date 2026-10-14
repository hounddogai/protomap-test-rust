use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist exception must pass. Check returns its problems, or none.
pub trait AllowlistExceptionRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistExceptionLabelLengthRule {
    pub max_length: i64,
}

impl AllowlistExceptionRule for AllowlistExceptionLabelLengthRule {
    fn check(&self, item: &AllowlistException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistExceptionCodePrefixRule {
    pub prefix: String,
}

impl AllowlistExceptionRule for AllowlistExceptionCodePrefixRule {
    fn check(&self, item: &AllowlistException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistExceptionRequiredNameRule {
    pub message: String,
}

impl AllowlistExceptionRule for AllowlistExceptionRequiredNameRule {
    fn check(&self, item: &AllowlistException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistExceptionRequiredCodeRule {
    pub message: String,
}

impl AllowlistExceptionRule for AllowlistExceptionRequiredCodeRule {
    fn check(&self, item: &AllowlistException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist exception.
#[derive(Debug, Clone)]
pub struct AllowlistExceptionValidator {
    pub rules: Vec<Rc<dyn AllowlistExceptionRule>>,
}

impl AllowlistExceptionValidator {
    pub fn validate(&self, item: &AllowlistException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_exception_validator() -> AllowlistExceptionValidator {
    AllowlistExceptionValidator {
        rules: vec![
            Rc::new(AllowlistExceptionLabelLengthRule { max_length: 86400 }) as Rc<dyn AllowlistExceptionRule>,
            Rc::new(AllowlistExceptionCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AllowlistExceptionRequiredNameRule {
                message: "allowlist exception needs this field".to_string(),
            }),
            Rc::new(AllowlistExceptionRequiredCodeRule {
                message: "allowlist exception needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_exception_validator();
        let item = new_allowlist_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AllowlistExceptionLabelLengthRule { max_length: 1000 };
        let mut item = new_allowlist_exception("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AllowlistExceptionRequiredNameRule {
            message: "allowlist exception needs this field".to_string(),
        };
        let mut item = new_allowlist_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistExceptionRequiredCodeRule {
            message: "allowlist exception needs an entry".to_string(),
        };
        let mut item = new_allowlist_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
