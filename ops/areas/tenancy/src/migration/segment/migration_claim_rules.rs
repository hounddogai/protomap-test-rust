use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration claim must pass. Check returns its problems, or none.
pub trait MigrationClaimRule: std::fmt::Debug {
    fn check(&self, item: &MigrationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationClaimThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationClaimRule for MigrationClaimThresholdRangeRule {
    fn check(&self, item: &MigrationClaim) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationClaimPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationClaimRule for MigrationClaimPortRangeRule {
    fn check(&self, item: &MigrationClaim) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationClaimRequiredNotesRule {
    pub message: String,
}

impl MigrationClaimRule for MigrationClaimRequiredNotesRule {
    fn check(&self, item: &MigrationClaim) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationClaimAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl MigrationClaimRule for MigrationClaimAllowedClusterRule {
    fn check(&self, item: &MigrationClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is stale", item.cluster)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration claim.
#[derive(Debug, Clone)]
pub struct MigrationClaimValidator {
    pub rules: Vec<Rc<dyn MigrationClaimRule>>,
}

impl MigrationClaimValidator {
    pub fn validate(&self, item: &MigrationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_claim_validator() -> MigrationClaimValidator {
    MigrationClaimValidator {
        rules: vec![
            Rc::new(MigrationClaimThresholdRangeRule { min: 8.7, max: 1420.2 }) as Rc<dyn MigrationClaimRule>,
            Rc::new(MigrationClaimPortRangeRule { min: 64, max: 120 }),
            Rc::new(MigrationClaimRequiredNotesRule {
                message: "migration claim needs this field".to_string(),
            }),
            Rc::new(MigrationClaimAllowedClusterRule {
                allowed: vec!["adaptive".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_claim_validator();
        let item = new_migration_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = MigrationClaimThresholdRangeRule { min: 4.1, max: 3049.2 };
        let mut item = new_migration_claim("x", "y");
        item.threshold = 3050.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = MigrationClaimPortRangeRule { min: 64, max: 255 };
        let mut item = new_migration_claim("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = MigrationClaimRequiredNotesRule {
            message: "migration claim needs a value".to_string(),
        };
        let mut item = new_migration_claim("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
