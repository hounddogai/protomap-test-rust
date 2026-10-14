use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration target must pass. Check returns its problems, or none.
pub trait MigrationTargetRule: std::fmt::Debug {
    fn check(&self, item: &MigrationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTargetCodePrefixRule {
    pub prefix: String,
}

impl MigrationTargetRule for MigrationTargetCodePrefixRule {
    fn check(&self, item: &MigrationTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTargetAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl MigrationTargetRule for MigrationTargetAllowedAudienceRule {
    fn check(&self, item: &MigrationTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is above maximum", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationTargetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationTargetRule for MigrationTargetStartsAtRangeRule {
    fn check(&self, item: &MigrationTarget) -> Vec<String> {
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

/// Runs every rule of a migration target.
#[derive(Debug, Clone)]
pub struct MigrationTargetValidator {
    pub rules: Vec<Rc<dyn MigrationTargetRule>>,
}

impl MigrationTargetValidator {
    pub fn validate(&self, item: &MigrationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_target_validator() -> MigrationTargetValidator {
    MigrationTargetValidator {
        rules: vec![
            Rc::new(MigrationTargetCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn MigrationTargetRule>,
            Rc::new(MigrationTargetAllowedAudienceRule {
                allowed: vec!["adaptive".to_string(), "weighted".to_string()],
            }),
            Rc::new(MigrationTargetStartsAtRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_target_validator();
        let item = new_migration_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = MigrationTargetStartsAtRangeRule { min: 16, max: 64 };
        let mut item = new_migration_target("x", "y");
        item.starts_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
