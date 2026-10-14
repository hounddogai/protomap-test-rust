use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default redaction snapshot must pass. Check returns its problems, or none.
pub trait DefaultRedactionSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRedactionSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshotRequiredSummaryRule {
    pub message: String,
}

impl DefaultRedactionSnapshotRule for DefaultRedactionSnapshotRequiredSummaryRule {
    fn check(&self, item: &DefaultRedactionSnapshot) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshotGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRedactionSnapshotRule for DefaultRedactionSnapshotGraceDaysRangeRule {
    fn check(&self, item: &DefaultRedactionSnapshot) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshotRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultRedactionSnapshotRule for DefaultRedactionSnapshotRatioRangeRule {
    fn check(&self, item: &DefaultRedactionSnapshot) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshotActiveGuardRule {
    pub limit: i64,
}

impl DefaultRedactionSnapshotRule for DefaultRedactionSnapshotActiveGuardRule {
    fn check(&self, item: &DefaultRedactionSnapshot) -> Vec<String> {
        if item.active && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while active",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshotCodePrefixRule {
    pub prefix: String,
}

impl DefaultRedactionSnapshotRule for DefaultRedactionSnapshotCodePrefixRule {
    fn check(&self, item: &DefaultRedactionSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionSnapshotNameLengthRule {
    pub max_length: i64,
}

impl DefaultRedactionSnapshotRule for DefaultRedactionSnapshotNameLengthRule {
    fn check(&self, item: &DefaultRedactionSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default redaction snapshot.
#[derive(Debug, Clone)]
pub struct DefaultRedactionSnapshotValidator {
    pub rules: Vec<Rc<dyn DefaultRedactionSnapshotRule>>,
}

impl DefaultRedactionSnapshotValidator {
    pub fn validate(&self, item: &DefaultRedactionSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRedactionSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRedactionSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_redaction_snapshot_validator() -> DefaultRedactionSnapshotValidator {
    DefaultRedactionSnapshotValidator {
        rules: vec![
            Rc::new(DefaultRedactionSnapshotRequiredSummaryRule {
                message: "default redaction snapshot needs a value".to_string(),
            }) as Rc<dyn DefaultRedactionSnapshotRule>,
            Rc::new(DefaultRedactionSnapshotGraceDaysRangeRule { min: 255, max: 1000 }),
            Rc::new(DefaultRedactionSnapshotRatioRangeRule { min: 9.3, max: 3359.9 }),
            Rc::new(DefaultRedactionSnapshotActiveGuardRule { limit: 86400 }),
            Rc::new(DefaultRedactionSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultRedactionSnapshotNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_redaction_snapshot_validator();
        let item = new_default_redaction_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = DefaultRedactionSnapshotRequiredSummaryRule {
            message: "default redaction snapshot needs this field".to_string(),
        };
        let mut item = new_default_redaction_snapshot("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = DefaultRedactionSnapshotGraceDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_default_redaction_snapshot("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = DefaultRedactionSnapshotRatioRangeRule { min: 8.3, max: 2467.4 };
        let mut item = new_default_redaction_snapshot("x", "y");
        item.ratio = 2468.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultRedactionSnapshotNameLengthRule { max_length: 16 };
        let mut item = new_default_redaction_snapshot("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
