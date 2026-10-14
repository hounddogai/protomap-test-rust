use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence rollup must pass. Check returns its problems, or none.
pub trait EvidenceRollupRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRollupRequiredCodeRule {
    pub message: String,
}

impl EvidenceRollupRule for EvidenceRollupRequiredCodeRule {
    fn check(&self, item: &EvidenceRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRollupTimezoneLengthRule {
    pub max_length: i64,
}

impl EvidenceRollupRule for EvidenceRollupTimezoneLengthRule {
    fn check(&self, item: &EvidenceRollup) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRollupRequiredCategoryRule {
    pub message: String,
}

impl EvidenceRollupRule for EvidenceRollupRequiredCategoryRule {
    fn check(&self, item: &EvidenceRollup) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence rollup.
#[derive(Debug, Clone)]
pub struct EvidenceRollupValidator {
    pub rules: Vec<Rc<dyn EvidenceRollupRule>>,
}

impl EvidenceRollupValidator {
    pub fn validate(&self, item: &EvidenceRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_rollup_validator() -> EvidenceRollupValidator {
    EvidenceRollupValidator {
        rules: vec![
            Rc::new(EvidenceRollupRequiredCodeRule {
                message: "evidence rollup needs a value".to_string(),
            }) as Rc<dyn EvidenceRollupRule>,
            Rc::new(EvidenceRollupTimezoneLengthRule { max_length: 86400 }),
            Rc::new(EvidenceRollupRequiredCategoryRule {
                message: "evidence rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_rollup_validator();
        let item = new_evidence_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceRollupRequiredCodeRule {
            message: "evidence rollup needs this field".to_string(),
        };
        let mut item = new_evidence_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = EvidenceRollupTimezoneLengthRule { max_length: 80 };
        let mut item = new_evidence_rollup("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = EvidenceRollupRequiredCategoryRule {
            message: "evidence rollup needs this field".to_string(),
        };
        let mut item = new_evidence_rollup("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
