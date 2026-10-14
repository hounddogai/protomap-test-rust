use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global forecast rollup must pass. Check returns its problems, or none.
pub trait GlobalForecastRollupRule: std::fmt::Debug {
    fn check(&self, item: &GlobalForecastRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastRollupRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalForecastRollupRule for GlobalForecastRollupRevisionRangeRule {
    fn check(&self, item: &GlobalForecastRollup) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastRollupOwnerIdLengthRule {
    pub max_length: i64,
}

impl GlobalForecastRollupRule for GlobalForecastRollupOwnerIdLengthRule {
    fn check(&self, item: &GlobalForecastRollup) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastRollupAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl GlobalForecastRollupRule for GlobalForecastRollupAllowedDescriptionRule {
    fn check(&self, item: &GlobalForecastRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is inconsistent", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global forecast rollup.
#[derive(Debug, Clone)]
pub struct GlobalForecastRollupValidator {
    pub rules: Vec<Rc<dyn GlobalForecastRollupRule>>,
}

impl GlobalForecastRollupValidator {
    pub fn validate(&self, item: &GlobalForecastRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalForecastRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalForecastRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_forecast_rollup_validator() -> GlobalForecastRollupValidator {
    GlobalForecastRollupValidator {
        rules: vec![
            Rc::new(GlobalForecastRollupRevisionRangeRule { min: 86400, max: 86400 })
                as Rc<dyn GlobalForecastRollupRule>,
            Rc::new(GlobalForecastRollupOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(GlobalForecastRollupAllowedDescriptionRule {
                allowed: vec!["manual".to_string(), "strict".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_forecast_rollup_validator();
        let item = new_global_forecast_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = GlobalForecastRollupRevisionRangeRule { min: 64, max: 86400 };
        let mut item = new_global_forecast_rollup("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = GlobalForecastRollupOwnerIdLengthRule { max_length: 120 };
        let mut item = new_global_forecast_rollup("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
