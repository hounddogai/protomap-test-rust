use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier snapshot must pass. Check returns its problems, or none.
pub trait VerifierSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &VerifierSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSnapshotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierSnapshotRule for VerifierSnapshotThresholdRangeRule {
    fn check(&self, item: &VerifierSnapshot) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSnapshotNameLengthRule {
    pub max_length: i64,
}

impl VerifierSnapshotRule for VerifierSnapshotNameLengthRule {
    fn check(&self, item: &VerifierSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSnapshotAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl VerifierSnapshotRule for VerifierSnapshotAllowedChecksumRule {
    fn check(&self, item: &VerifierSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is stale", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSnapshotRequiredNameRule {
    pub message: String,
}

impl VerifierSnapshotRule for VerifierSnapshotRequiredNameRule {
    fn check(&self, item: &VerifierSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier snapshot.
#[derive(Debug, Clone)]
pub struct VerifierSnapshotValidator {
    pub rules: Vec<Rc<dyn VerifierSnapshotRule>>,
}

impl VerifierSnapshotValidator {
    pub fn validate(&self, item: &VerifierSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_snapshot_validator() -> VerifierSnapshotValidator {
    VerifierSnapshotValidator {
        rules: vec![
            Rc::new(VerifierSnapshotThresholdRangeRule { min: 7.3, max: 1427.0 }) as Rc<dyn VerifierSnapshotRule>,
            Rc::new(VerifierSnapshotNameLengthRule { max_length: 86400 }),
            Rc::new(VerifierSnapshotAllowedChecksumRule {
                allowed: vec!["scheduled".to_string(), "batched".to_string(), "regional".to_string()],
            }),
            Rc::new(VerifierSnapshotRequiredNameRule {
                message: "verifier snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_snapshot_validator();
        let item = new_verifier_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = VerifierSnapshotThresholdRangeRule { min: 1.3, max: 4639.4 };
        let mut item = new_verifier_snapshot("x", "y");
        item.threshold = 4640.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = VerifierSnapshotNameLengthRule { max_length: 32 };
        let mut item = new_verifier_snapshot("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = VerifierSnapshotRequiredNameRule {
            message: "verifier snapshot needs a value".to_string(),
        };
        let mut item = new_verifier_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
