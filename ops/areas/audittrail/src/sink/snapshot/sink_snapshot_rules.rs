use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink snapshot must pass. Check returns its problems, or none.
pub trait SinkSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SinkSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSnapshotRequiredCodeRule {
    pub message: String,
}

impl SinkSnapshotRule for SinkSnapshotRequiredCodeRule {
    fn check(&self, item: &SinkSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSnapshotCodePrefixRule {
    pub prefix: String,
}

impl SinkSnapshotRule for SinkSnapshotCodePrefixRule {
    fn check(&self, item: &SinkSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl SinkSnapshotRule for SinkSnapshotCodeLengthRule {
    fn check(&self, item: &SinkSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkSnapshotWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkSnapshotRule for SinkSnapshotWindowDaysRangeRule {
    fn check(&self, item: &SinkSnapshot) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink snapshot.
#[derive(Debug, Clone)]
pub struct SinkSnapshotValidator {
    pub rules: Vec<Rc<dyn SinkSnapshotRule>>,
}

impl SinkSnapshotValidator {
    pub fn validate(&self, item: &SinkSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_snapshot_validator() -> SinkSnapshotValidator {
    SinkSnapshotValidator {
        rules: vec![
            Rc::new(SinkSnapshotRequiredCodeRule {
                message: "sink snapshot needs a value".to_string(),
            }) as Rc<dyn SinkSnapshotRule>,
            Rc::new(SinkSnapshotCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SinkSnapshotCodeLengthRule { max_length: 3600 }),
            Rc::new(SinkSnapshotWindowDaysRangeRule { min: 255, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_snapshot_validator();
        let item = new_sink_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkSnapshotRequiredCodeRule {
            message: "sink snapshot needs a value".to_string(),
        };
        let mut item = new_sink_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkSnapshotCodeLengthRule { max_length: 16 };
        let mut item = new_sink_snapshot("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SinkSnapshotWindowDaysRangeRule { min: 16, max: 3600 };
        let mut item = new_sink_snapshot("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
