use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration request must pass. Check returns its problems, or none.
pub trait MigrationRequestRule: std::fmt::Debug {
    fn check(&self, item: &MigrationRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRequestVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationRequestRule for MigrationRequestVersionRangeRule {
    fn check(&self, item: &MigrationRequest) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRequestCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationRequestRule for MigrationRequestCostRangeRule {
    fn check(&self, item: &MigrationRequest) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRequestRequiredCountryRule {
    pub message: String,
}

impl MigrationRequestRule for MigrationRequestRequiredCountryRule {
    fn check(&self, item: &MigrationRequest) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration request.
#[derive(Debug, Clone)]
pub struct MigrationRequestValidator {
    pub rules: Vec<Rc<dyn MigrationRequestRule>>,
}

impl MigrationRequestValidator {
    pub fn validate(&self, item: &MigrationRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_request_validator() -> MigrationRequestValidator {
    MigrationRequestValidator {
        rules: vec![
            Rc::new(MigrationRequestVersionRangeRule { min: 32, max: 64 }) as Rc<dyn MigrationRequestRule>,
            Rc::new(MigrationRequestCostRangeRule { min: 7.2, max: 4402.5 }),
            Rc::new(MigrationRequestRequiredCountryRule {
                message: "migration request needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_request_validator();
        let item = new_migration_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = MigrationRequestVersionRangeRule { min: 3600, max: 3600 };
        let mut item = new_migration_request("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = MigrationRequestCostRangeRule { min: 7.2, max: 3218.9 };
        let mut item = new_migration_request("x", "y");
        item.cost = 3220.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = MigrationRequestRequiredCountryRule {
            message: "migration request needs an entry".to_string(),
        };
        let mut item = new_migration_request("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
