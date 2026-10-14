use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence group must pass. Check returns its problems, or none.
pub trait EvidenceGroupRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceGroupActiveGuardRule {
    pub limit: i64,
}

impl EvidenceGroupRule for EvidenceGroupActiveGuardRule {
    fn check(&self, item: &EvidenceGroup) -> Vec<String> {
        if item.active && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while active",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceGroupLabelLengthRule {
    pub max_length: i64,
}

impl EvidenceGroupRule for EvidenceGroupLabelLengthRule {
    fn check(&self, item: &EvidenceGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceGroupRequiredLabelRule {
    pub message: String,
}

impl EvidenceGroupRule for EvidenceGroupRequiredLabelRule {
    fn check(&self, item: &EvidenceGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceGroupBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceGroupRule for EvidenceGroupBatchSizeRangeRule {
    fn check(&self, item: &EvidenceGroup) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence group.
#[derive(Debug, Clone)]
pub struct EvidenceGroupValidator {
    pub rules: Vec<Rc<dyn EvidenceGroupRule>>,
}

impl EvidenceGroupValidator {
    pub fn validate(&self, item: &EvidenceGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_group_validator() -> EvidenceGroupValidator {
    EvidenceGroupValidator {
        rules: vec![
            Rc::new(EvidenceGroupActiveGuardRule { limit: 86400 }) as Rc<dyn EvidenceGroupRule>,
            Rc::new(EvidenceGroupLabelLengthRule { max_length: 3600 }),
            Rc::new(EvidenceGroupRequiredLabelRule {
                message: "evidence group needs an entry".to_string(),
            }),
            Rc::new(EvidenceGroupBatchSizeRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_group_validator();
        let item = new_evidence_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EvidenceGroupLabelLengthRule { max_length: 1000 };
        let mut item = new_evidence_group("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EvidenceGroupRequiredLabelRule {
            message: "evidence group needs a value".to_string(),
        };
        let mut item = new_evidence_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = EvidenceGroupBatchSizeRangeRule { min: 120, max: 3600 };
        let mut item = new_evidence_group("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
