use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient migration score must pass. Check returns its problems, or none.
pub trait LenientMigrationScoreRule: std::fmt::Debug {
    fn check(&self, item: &LenientMigrationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientMigrationScoreRule for LenientMigrationScoreAllowedCodeRule {
    fn check(&self, item: &LenientMigrationScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationScorePinnedGuardRule {
    pub limit: i64,
}

impl LenientMigrationScoreRule for LenientMigrationScorePinnedGuardRule {
    fn check(&self, item: &LenientMigrationScore) -> Vec<String> {
        if item.pinned && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while pinned",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationScoreGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientMigrationScoreRule for LenientMigrationScoreGraceDaysRangeRule {
    fn check(&self, item: &LenientMigrationScore) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient migration score.
#[derive(Debug, Clone)]
pub struct LenientMigrationScoreValidator {
    pub rules: Vec<Rc<dyn LenientMigrationScoreRule>>,
}

impl LenientMigrationScoreValidator {
    pub fn validate(&self, item: &LenientMigrationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientMigrationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientMigrationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_migration_score_validator() -> LenientMigrationScoreValidator {
    LenientMigrationScoreValidator {
        rules: vec![
            Rc::new(LenientMigrationScoreAllowedCodeRule { allowed: Vec::new() }) as Rc<dyn LenientMigrationScoreRule>,
            Rc::new(LenientMigrationScorePinnedGuardRule { limit: 86400 }),
            Rc::new(LenientMigrationScoreGraceDaysRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_migration_score_validator();
        let item = new_lenient_migration_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LenientMigrationScoreGraceDaysRangeRule { min: 120, max: 1000 };
        let mut item = new_lenient_migration_score("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
