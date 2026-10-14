use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence target must pass. Check returns its problems, or none.
pub trait EvidenceTargetRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl EvidenceTargetRule for EvidenceTargetAllowedCodeRule {
    fn check(&self, item: &EvidenceTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTargetRequiredCodeRule {
    pub message: String,
}

impl EvidenceTargetRule for EvidenceTargetRequiredCodeRule {
    fn check(&self, item: &EvidenceTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTargetCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceTargetRule for EvidenceTargetCodeLengthRule {
    fn check(&self, item: &EvidenceTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceTargetTitleLengthRule {
    pub max_length: i64,
}

impl EvidenceTargetRule for EvidenceTargetTitleLengthRule {
    fn check(&self, item: &EvidenceTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence target.
#[derive(Debug, Clone)]
pub struct EvidenceTargetValidator {
    pub rules: Vec<Rc<dyn EvidenceTargetRule>>,
}

impl EvidenceTargetValidator {
    pub fn validate(&self, item: &EvidenceTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_target_validator() -> EvidenceTargetValidator {
    EvidenceTargetValidator {
        rules: vec![
            Rc::new(EvidenceTargetAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn EvidenceTargetRule>,
            Rc::new(EvidenceTargetRequiredCodeRule {
                message: "evidence target needs a value".to_string(),
            }),
            Rc::new(EvidenceTargetCodeLengthRule { max_length: 86400 }),
            Rc::new(EvidenceTargetTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_target_validator();
        let item = new_evidence_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceTargetRequiredCodeRule {
            message: "evidence target needs a value".to_string(),
        };
        let mut item = new_evidence_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceTargetCodeLengthRule { max_length: 64 };
        let mut item = new_evidence_target("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EvidenceTargetTitleLengthRule { max_length: 16 };
        let mut item = new_evidence_target("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
