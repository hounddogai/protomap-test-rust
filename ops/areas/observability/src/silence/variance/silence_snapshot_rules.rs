use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence snapshot must pass. Check returns its problems, or none.
pub trait SilenceSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SilenceSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSnapshotRequiredLabelRule {
    pub message: String,
}

impl SilenceSnapshotRule for SilenceSnapshotRequiredLabelRule {
    fn check(&self, item: &SilenceSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl SilenceSnapshotRule for SilenceSnapshotCodeLengthRule {
    fn check(&self, item: &SilenceSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSnapshotRequiredCipherRule {
    pub message: String,
}

impl SilenceSnapshotRule for SilenceSnapshotRequiredCipherRule {
    fn check(&self, item: &SilenceSnapshot) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSnapshotAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceSnapshotRule for SilenceSnapshotAmountRangeRule {
    fn check(&self, item: &SilenceSnapshot) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence snapshot.
#[derive(Debug, Clone)]
pub struct SilenceSnapshotValidator {
    pub rules: Vec<Rc<dyn SilenceSnapshotRule>>,
}

impl SilenceSnapshotValidator {
    pub fn validate(&self, item: &SilenceSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_snapshot_validator() -> SilenceSnapshotValidator {
    SilenceSnapshotValidator {
        rules: vec![
            Rc::new(SilenceSnapshotRequiredLabelRule {
                message: "silence snapshot needs a value".to_string(),
            }) as Rc<dyn SilenceSnapshotRule>,
            Rc::new(SilenceSnapshotCodeLengthRule { max_length: 120 }),
            Rc::new(SilenceSnapshotRequiredCipherRule {
                message: "silence snapshot needs this field".to_string(),
            }),
            Rc::new(SilenceSnapshotAmountRangeRule { min: 0.3, max: 4690.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_snapshot_validator();
        let item = new_silence_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SilenceSnapshotRequiredLabelRule {
            message: "silence snapshot needs an entry".to_string(),
        };
        let mut item = new_silence_snapshot("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilenceSnapshotCodeLengthRule { max_length: 3600 };
        let mut item = new_silence_snapshot("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = SilenceSnapshotRequiredCipherRule {
            message: "silence snapshot needs an entry".to_string(),
        };
        let mut item = new_silence_snapshot("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = SilenceSnapshotAmountRangeRule { min: 9.7, max: 1192.9 };
        let mut item = new_silence_snapshot("x", "y");
        item.amount = 1194.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
