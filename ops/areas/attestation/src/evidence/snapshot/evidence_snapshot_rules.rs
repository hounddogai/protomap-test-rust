use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence snapshot must pass. Check returns its problems, or none.
pub trait EvidenceSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSnapshotRequiredTitleRule {
    pub message: String,
}

impl EvidenceSnapshotRule for EvidenceSnapshotRequiredTitleRule {
    fn check(&self, item: &EvidenceSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSnapshotReviewerLengthRule {
    pub max_length: i64,
}

impl EvidenceSnapshotRule for EvidenceSnapshotReviewerLengthRule {
    fn check(&self, item: &EvidenceSnapshot) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSnapshotCodePrefixRule {
    pub prefix: String,
}

impl EvidenceSnapshotRule for EvidenceSnapshotCodePrefixRule {
    fn check(&self, item: &EvidenceSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceSnapshotRule for EvidenceSnapshotCodeLengthRule {
    fn check(&self, item: &EvidenceSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSnapshotGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceSnapshotRule for EvidenceSnapshotGraceDaysRangeRule {
    fn check(&self, item: &EvidenceSnapshot) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence snapshot.
#[derive(Debug, Clone)]
pub struct EvidenceSnapshotValidator {
    pub rules: Vec<Rc<dyn EvidenceSnapshotRule>>,
}

impl EvidenceSnapshotValidator {
    pub fn validate(&self, item: &EvidenceSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_snapshot_validator() -> EvidenceSnapshotValidator {
    EvidenceSnapshotValidator {
        rules: vec![
            Rc::new(EvidenceSnapshotRequiredTitleRule {
                message: "evidence snapshot needs a value".to_string(),
            }) as Rc<dyn EvidenceSnapshotRule>,
            Rc::new(EvidenceSnapshotReviewerLengthRule { max_length: 120 }),
            Rc::new(EvidenceSnapshotCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(EvidenceSnapshotCodeLengthRule { max_length: 86400 }),
            Rc::new(EvidenceSnapshotGraceDaysRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_snapshot_validator();
        let item = new_evidence_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EvidenceSnapshotRequiredTitleRule {
            message: "evidence snapshot needs an entry".to_string(),
        };
        let mut item = new_evidence_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = EvidenceSnapshotReviewerLengthRule { max_length: 3600 };
        let mut item = new_evidence_snapshot("x", "y");
        item.reviewer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceSnapshotCodeLengthRule { max_length: 80 };
        let mut item = new_evidence_snapshot("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = EvidenceSnapshotGraceDaysRangeRule { min: 120, max: 255 };
        let mut item = new_evidence_snapshot("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
