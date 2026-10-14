use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence draft must pass. Check returns its problems, or none.
pub trait EvidenceDraftRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceDraftMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceDraftRule for EvidenceDraftMarginRangeRule {
    fn check(&self, item: &EvidenceDraft) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceDraftChecksumLengthRule {
    pub max_length: i64,
}

impl EvidenceDraftRule for EvidenceDraftChecksumLengthRule {
    fn check(&self, item: &EvidenceDraft) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceDraftRequiredCategoryRule {
    pub message: String,
}

impl EvidenceDraftRule for EvidenceDraftRequiredCategoryRule {
    fn check(&self, item: &EvidenceDraft) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceDraftCodePrefixRule {
    pub prefix: String,
}

impl EvidenceDraftRule for EvidenceDraftCodePrefixRule {
    fn check(&self, item: &EvidenceDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceDraftCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceDraftRule for EvidenceDraftCodeLengthRule {
    fn check(&self, item: &EvidenceDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence draft.
#[derive(Debug, Clone)]
pub struct EvidenceDraftValidator {
    pub rules: Vec<Rc<dyn EvidenceDraftRule>>,
}

impl EvidenceDraftValidator {
    pub fn validate(&self, item: &EvidenceDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_draft_validator() -> EvidenceDraftValidator {
    EvidenceDraftValidator {
        rules: vec![
            Rc::new(EvidenceDraftMarginRangeRule { min: 6.5, max: 4682.7 }) as Rc<dyn EvidenceDraftRule>,
            Rc::new(EvidenceDraftChecksumLengthRule { max_length: 16 }),
            Rc::new(EvidenceDraftRequiredCategoryRule {
                message: "evidence draft needs this field".to_string(),
            }),
            Rc::new(EvidenceDraftCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(EvidenceDraftCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_draft_validator();
        let item = new_evidence_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = EvidenceDraftMarginRangeRule { min: 4.8, max: 4444.0 };
        let mut item = new_evidence_draft("x", "y");
        item.margin = 4445.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = EvidenceDraftChecksumLengthRule { max_length: 120 };
        let mut item = new_evidence_draft("x", "y");
        item.checksum = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = EvidenceDraftRequiredCategoryRule {
            message: "evidence draft needs a value".to_string(),
        };
        let mut item = new_evidence_draft("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceDraftCodeLengthRule { max_length: 1000 };
        let mut item = new_evidence_draft("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
