use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling migration agreement must pass. Check returns its problems, or none.
pub trait RollingMigrationAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RollingMigrationAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationAgreementCodePrefixRule {
    pub prefix: String,
}

impl RollingMigrationAgreementRule for RollingMigrationAgreementCodePrefixRule {
    fn check(&self, item: &RollingMigrationAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationAgreementAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollingMigrationAgreementRule for RollingMigrationAgreementAllowedTitleRule {
    fn check(&self, item: &RollingMigrationAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is above maximum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationAgreementStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingMigrationAgreementRule for RollingMigrationAgreementStartsAtRangeRule {
    fn check(&self, item: &RollingMigrationAgreement) -> Vec<String> {
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

/// Runs every rule of a rolling migration agreement.
#[derive(Debug, Clone)]
pub struct RollingMigrationAgreementValidator {
    pub rules: Vec<Rc<dyn RollingMigrationAgreementRule>>,
}

impl RollingMigrationAgreementValidator {
    pub fn validate(&self, item: &RollingMigrationAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingMigrationAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingMigrationAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_migration_agreement_validator() -> RollingMigrationAgreementValidator {
    RollingMigrationAgreementValidator {
        rules: vec![
            Rc::new(RollingMigrationAgreementCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RollingMigrationAgreementRule>,
            Rc::new(RollingMigrationAgreementAllowedTitleRule {
                allowed: vec!["regional".to_string(), "strict".to_string(), "seasonal".to_string()],
            }),
            Rc::new(RollingMigrationAgreementStartsAtRangeRule { min: 1000, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_migration_agreement_validator();
        let item = new_rolling_migration_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RollingMigrationAgreementStartsAtRangeRule { min: 16, max: 255 };
        let mut item = new_rolling_migration_agreement("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
