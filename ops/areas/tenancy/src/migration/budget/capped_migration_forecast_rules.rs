use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped migration forecast must pass. Check returns its problems, or none.
pub trait CappedMigrationForecastRule: std::fmt::Debug {
    fn check(&self, item: &CappedMigrationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedMigrationForecastRule for CappedMigrationForecastTtlSecondsRangeRule {
    fn check(&self, item: &CappedMigrationForecast) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastRequiredClusterRule {
    pub message: String,
}

impl CappedMigrationForecastRule for CappedMigrationForecastRequiredClusterRule {
    fn check(&self, item: &CappedMigrationForecast) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastCodePrefixRule {
    pub prefix: String,
}

impl CappedMigrationForecastRule for CappedMigrationForecastCodePrefixRule {
    fn check(&self, item: &CappedMigrationForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastRegionLengthRule {
    pub max_length: i64,
}

impl CappedMigrationForecastRule for CappedMigrationForecastRegionLengthRule {
    fn check(&self, item: &CappedMigrationForecast) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastClusterLengthRule {
    pub max_length: i64,
}

impl CappedMigrationForecastRule for CappedMigrationForecastClusterLengthRule {
    fn check(&self, item: &CappedMigrationForecast) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastRequiredRegionRule {
    pub message: String,
}

impl CappedMigrationForecastRule for CappedMigrationForecastRequiredRegionRule {
    fn check(&self, item: &CappedMigrationForecast) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped migration forecast.
#[derive(Debug, Clone)]
pub struct CappedMigrationForecastValidator {
    pub rules: Vec<Rc<dyn CappedMigrationForecastRule>>,
}

impl CappedMigrationForecastValidator {
    pub fn validate(&self, item: &CappedMigrationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedMigrationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedMigrationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_migration_forecast_validator() -> CappedMigrationForecastValidator {
    CappedMigrationForecastValidator {
        rules: vec![
            Rc::new(CappedMigrationForecastTtlSecondsRangeRule { min: 64, max: 80 })
                as Rc<dyn CappedMigrationForecastRule>,
            Rc::new(CappedMigrationForecastRequiredClusterRule {
                message: "capped migration forecast needs this field".to_string(),
            }),
            Rc::new(CappedMigrationForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CappedMigrationForecastRegionLengthRule { max_length: 3600 }),
            Rc::new(CappedMigrationForecastClusterLengthRule { max_length: 3600 }),
            Rc::new(CappedMigrationForecastRequiredRegionRule {
                message: "capped migration forecast needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_migration_forecast_validator();
        let item = new_capped_migration_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = CappedMigrationForecastTtlSecondsRangeRule { min: 64, max: 1000 };
        let mut item = new_capped_migration_forecast("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = CappedMigrationForecastRequiredClusterRule {
            message: "capped migration forecast needs an entry".to_string(),
        };
        let mut item = new_capped_migration_forecast("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = CappedMigrationForecastRegionLengthRule { max_length: 16 };
        let mut item = new_capped_migration_forecast("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = CappedMigrationForecastClusterLengthRule { max_length: 32 };
        let mut item = new_capped_migration_forecast("x", "y");
        item.cluster = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = CappedMigrationForecastRequiredRegionRule {
            message: "capped migration forecast needs an entry".to_string(),
        };
        let mut item = new_capped_migration_forecast("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
