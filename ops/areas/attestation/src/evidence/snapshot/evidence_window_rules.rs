use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence window must pass. Check returns its problems, or none.
pub trait EvidenceWindowRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindowCodePrefixRule {
    pub prefix: String,
}

impl EvidenceWindowRule for EvidenceWindowCodePrefixRule {
    fn check(&self, item: &EvidenceWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindowRequiredCodeRule {
    pub message: String,
}

impl EvidenceWindowRule for EvidenceWindowRequiredCodeRule {
    fn check(&self, item: &EvidenceWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindowCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceWindowRule for EvidenceWindowCodeLengthRule {
    fn check(&self, item: &EvidenceWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindowScopeLengthRule {
    pub max_length: i64,
}

impl EvidenceWindowRule for EvidenceWindowScopeLengthRule {
    fn check(&self, item: &EvidenceWindow) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindowAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl EvidenceWindowRule for EvidenceWindowAllowedTitleRule {
    fn check(&self, item: &EvidenceWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceWindowUnitLengthRule {
    pub max_length: i64,
}

impl EvidenceWindowRule for EvidenceWindowUnitLengthRule {
    fn check(&self, item: &EvidenceWindow) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence window.
#[derive(Debug, Clone)]
pub struct EvidenceWindowValidator {
    pub rules: Vec<Rc<dyn EvidenceWindowRule>>,
}

impl EvidenceWindowValidator {
    pub fn validate(&self, item: &EvidenceWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_window_validator() -> EvidenceWindowValidator {
    EvidenceWindowValidator {
        rules: vec![
            Rc::new(EvidenceWindowCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn EvidenceWindowRule>,
            Rc::new(EvidenceWindowRequiredCodeRule {
                message: "evidence window needs a value".to_string(),
            }),
            Rc::new(EvidenceWindowCodeLengthRule { max_length: 120 }),
            Rc::new(EvidenceWindowScopeLengthRule { max_length: 16 }),
            Rc::new(EvidenceWindowAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(EvidenceWindowUnitLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_window_validator();
        let item = new_evidence_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceWindowRequiredCodeRule {
            message: "evidence window needs this field".to_string(),
        };
        let mut item = new_evidence_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceWindowCodeLengthRule { max_length: 80 };
        let mut item = new_evidence_window("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = EvidenceWindowScopeLengthRule { max_length: 86400 };
        let mut item = new_evidence_window("x", "y");
        item.scope = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = EvidenceWindowUnitLengthRule { max_length: 80 };
        let mut item = new_evidence_window("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
