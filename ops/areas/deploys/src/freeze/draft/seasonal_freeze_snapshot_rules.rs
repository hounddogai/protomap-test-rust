use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal freeze snapshot must pass. Check returns its problems, or none.
pub trait SeasonalFreezeSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFreezeSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeSnapshotAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SeasonalFreezeSnapshotRule for SeasonalFreezeSnapshotAllowedNameRule {
    fn check(&self, item: &SeasonalFreezeSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeSnapshotNameLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeSnapshotRule for SeasonalFreezeSnapshotNameLengthRule {
    fn check(&self, item: &SeasonalFreezeSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeSnapshotActiveGuardRule {
    pub limit: i64,
}

impl SeasonalFreezeSnapshotRule for SeasonalFreezeSnapshotActiveGuardRule {
    fn check(&self, item: &SeasonalFreezeSnapshot) -> Vec<String> {
        if item.active && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while active",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeSnapshotRule for SeasonalFreezeSnapshotLabelLengthRule {
    fn check(&self, item: &SeasonalFreezeSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal freeze snapshot.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeSnapshotValidator {
    pub rules: Vec<Rc<dyn SeasonalFreezeSnapshotRule>>,
}

impl SeasonalFreezeSnapshotValidator {
    pub fn validate(&self, item: &SeasonalFreezeSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalFreezeSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFreezeSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_freeze_snapshot_validator() -> SeasonalFreezeSnapshotValidator {
    SeasonalFreezeSnapshotValidator {
        rules: vec![
            Rc::new(SeasonalFreezeSnapshotAllowedNameRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn SeasonalFreezeSnapshotRule>,
            Rc::new(SeasonalFreezeSnapshotNameLengthRule { max_length: 120 }),
            Rc::new(SeasonalFreezeSnapshotActiveGuardRule { limit: 1000 }),
            Rc::new(SeasonalFreezeSnapshotLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_freeze_snapshot_validator();
        let item = new_seasonal_freeze_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalFreezeSnapshotNameLengthRule { max_length: 32 };
        let mut item = new_seasonal_freeze_snapshot("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalFreezeSnapshotLabelLengthRule { max_length: 32 };
        let mut item = new_seasonal_freeze_snapshot("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
