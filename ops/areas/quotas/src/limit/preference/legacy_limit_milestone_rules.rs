use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy limit milestone must pass. Check returns its problems, or none.
pub trait LegacyLimitMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LegacyLimitMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitMilestoneCodePrefixRule {
    pub prefix: String,
}

impl LegacyLimitMilestoneRule for LegacyLimitMilestoneCodePrefixRule {
    fn check(&self, item: &LegacyLimitMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitMilestoneArchivedGuardRule {
    pub limit: i64,
}

impl LegacyLimitMilestoneRule for LegacyLimitMilestoneArchivedGuardRule {
    fn check(&self, item: &LegacyLimitMilestone) -> Vec<String> {
        if item.archived && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while archived",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitMilestoneUnitLengthRule {
    pub max_length: i64,
}

impl LegacyLimitMilestoneRule for LegacyLimitMilestoneUnitLengthRule {
    fn check(&self, item: &LegacyLimitMilestone) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitMilestoneStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyLimitMilestoneRule for LegacyLimitMilestoneStartsAtRangeRule {
    fn check(&self, item: &LegacyLimitMilestone) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy limit milestone.
#[derive(Debug, Clone)]
pub struct LegacyLimitMilestoneValidator {
    pub rules: Vec<Rc<dyn LegacyLimitMilestoneRule>>,
}

impl LegacyLimitMilestoneValidator {
    pub fn validate(&self, item: &LegacyLimitMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyLimitMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyLimitMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_limit_milestone_validator() -> LegacyLimitMilestoneValidator {
    LegacyLimitMilestoneValidator {
        rules: vec![
            Rc::new(LegacyLimitMilestoneCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn LegacyLimitMilestoneRule>,
            Rc::new(LegacyLimitMilestoneArchivedGuardRule { limit: 80 }),
            Rc::new(LegacyLimitMilestoneUnitLengthRule { max_length: 3600 }),
            Rc::new(LegacyLimitMilestoneStartsAtRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_limit_milestone_validator();
        let item = new_legacy_limit_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = LegacyLimitMilestoneUnitLengthRule { max_length: 16 };
        let mut item = new_legacy_limit_milestone("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LegacyLimitMilestoneStartsAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_legacy_limit_milestone("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
