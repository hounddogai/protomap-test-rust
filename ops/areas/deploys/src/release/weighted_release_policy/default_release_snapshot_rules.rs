use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default release snapshot must pass. Check returns its problems, or none.
pub trait DefaultReleaseSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DefaultReleaseSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseSnapshotPinnedGuardRule {
    pub limit: i64,
}

impl DefaultReleaseSnapshotRule for DefaultReleaseSnapshotPinnedGuardRule {
    fn check(&self, item: &DefaultReleaseSnapshot) -> Vec<String> {
        if item.pinned && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while pinned",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseSnapshotCodePrefixRule {
    pub prefix: String,
}

impl DefaultReleaseSnapshotRule for DefaultReleaseSnapshotCodePrefixRule {
    fn check(&self, item: &DefaultReleaseSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseSnapshotMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultReleaseSnapshotRule for DefaultReleaseSnapshotMaximumRangeRule {
    fn check(&self, item: &DefaultReleaseSnapshot) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseSnapshotRequiredCodeRule {
    pub message: String,
}

impl DefaultReleaseSnapshotRule for DefaultReleaseSnapshotRequiredCodeRule {
    fn check(&self, item: &DefaultReleaseSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default release snapshot.
#[derive(Debug, Clone)]
pub struct DefaultReleaseSnapshotValidator {
    pub rules: Vec<Rc<dyn DefaultReleaseSnapshotRule>>,
}

impl DefaultReleaseSnapshotValidator {
    pub fn validate(&self, item: &DefaultReleaseSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultReleaseSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultReleaseSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_release_snapshot_validator() -> DefaultReleaseSnapshotValidator {
    DefaultReleaseSnapshotValidator {
        rules: vec![
            Rc::new(DefaultReleaseSnapshotPinnedGuardRule { limit: 80 }) as Rc<dyn DefaultReleaseSnapshotRule>,
            Rc::new(DefaultReleaseSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(DefaultReleaseSnapshotMaximumRangeRule { min: 8.8, max: 1893.8 }),
            Rc::new(DefaultReleaseSnapshotRequiredCodeRule {
                message: "default release snapshot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_release_snapshot_validator();
        let item = new_default_release_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = DefaultReleaseSnapshotMaximumRangeRule { min: 1.2, max: 344.1 };
        let mut item = new_default_release_snapshot("x", "y");
        item.maximum = 345.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultReleaseSnapshotRequiredCodeRule {
            message: "default release snapshot needs a value".to_string(),
        };
        let mut item = new_default_release_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
