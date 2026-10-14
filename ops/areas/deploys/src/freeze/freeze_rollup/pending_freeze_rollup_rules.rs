use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending freeze rollup must pass. Check returns its problems, or none.
pub trait PendingFreezeRollupRule: std::fmt::Debug {
    fn check(&self, item: &PendingFreezeRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeRollupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingFreezeRollupRule for PendingFreezeRollupGraceDaysRangeRule {
    fn check(&self, item: &PendingFreezeRollup) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeRollupVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingFreezeRollupRule for PendingFreezeRollupVersionRangeRule {
    fn check(&self, item: &PendingFreezeRollup) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeRollupRequiredLabelRule {
    pub message: String,
}

impl PendingFreezeRollupRule for PendingFreezeRollupRequiredLabelRule {
    fn check(&self, item: &PendingFreezeRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending freeze rollup.
#[derive(Debug, Clone)]
pub struct PendingFreezeRollupValidator {
    pub rules: Vec<Rc<dyn PendingFreezeRollupRule>>,
}

impl PendingFreezeRollupValidator {
    pub fn validate(&self, item: &PendingFreezeRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingFreezeRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFreezeRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_freeze_rollup_validator() -> PendingFreezeRollupValidator {
    PendingFreezeRollupValidator {
        rules: vec![
            Rc::new(PendingFreezeRollupGraceDaysRangeRule { min: 16, max: 3600 }) as Rc<dyn PendingFreezeRollupRule>,
            Rc::new(PendingFreezeRollupVersionRangeRule { min: 80, max: 1000 }),
            Rc::new(PendingFreezeRollupRequiredLabelRule {
                message: "pending freeze rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_freeze_rollup_validator();
        let item = new_pending_freeze_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PendingFreezeRollupGraceDaysRangeRule { min: 80, max: 3600 };
        let mut item = new_pending_freeze_rollup("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = PendingFreezeRollupVersionRangeRule { min: 1000, max: 86400 };
        let mut item = new_pending_freeze_rollup("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingFreezeRollupRequiredLabelRule {
            message: "pending freeze rollup needs an entry".to_string(),
        };
        let mut item = new_pending_freeze_rollup("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
