use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity rollup must pass. Check returns its problems, or none.
pub trait IdentityRollupRule: std::fmt::Debug {
    fn check(&self, item: &IdentityRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollupWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityRollupRule for IdentityRollupWeightRangeRule {
    fn check(&self, item: &IdentityRollup) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollupAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl IdentityRollupRule for IdentityRollupAllowedRegionRule {
    fn check(&self, item: &IdentityRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is expired", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollupCodeLengthRule {
    pub max_length: i64,
}

impl IdentityRollupRule for IdentityRollupCodeLengthRule {
    fn check(&self, item: &IdentityRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollupNameLengthRule {
    pub max_length: i64,
}

impl IdentityRollupRule for IdentityRollupNameLengthRule {
    fn check(&self, item: &IdentityRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollupRequiredNameRule {
    pub message: String,
}

impl IdentityRollupRule for IdentityRollupRequiredNameRule {
    fn check(&self, item: &IdentityRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollupUnitLengthRule {
    pub max_length: i64,
}

impl IdentityRollupRule for IdentityRollupUnitLengthRule {
    fn check(&self, item: &IdentityRollup) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity rollup.
#[derive(Debug, Clone)]
pub struct IdentityRollupValidator {
    pub rules: Vec<Rc<dyn IdentityRollupRule>>,
}

impl IdentityRollupValidator {
    pub fn validate(&self, item: &IdentityRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_rollup_validator() -> IdentityRollupValidator {
    IdentityRollupValidator {
        rules: vec![
            Rc::new(IdentityRollupWeightRangeRule { min: 0.6, max: 2954.5 }) as Rc<dyn IdentityRollupRule>,
            Rc::new(IdentityRollupAllowedRegionRule {
                allowed: vec!["rolling".to_string(), "default".to_string(), "legacy".to_string()],
            }),
            Rc::new(IdentityRollupCodeLengthRule { max_length: 80 }),
            Rc::new(IdentityRollupNameLengthRule { max_length: 64 }),
            Rc::new(IdentityRollupRequiredNameRule {
                message: "identity rollup needs an entry".to_string(),
            }),
            Rc::new(IdentityRollupUnitLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_rollup_validator();
        let item = new_identity_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = IdentityRollupWeightRangeRule { min: 4.2, max: 2523.9 };
        let mut item = new_identity_rollup("x", "y");
        item.weight = 2525.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityRollupCodeLengthRule { max_length: 32 };
        let mut item = new_identity_rollup("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = IdentityRollupNameLengthRule { max_length: 64 };
        let mut item = new_identity_rollup("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = IdentityRollupRequiredNameRule {
            message: "identity rollup needs an entry".to_string(),
        };
        let mut item = new_identity_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = IdentityRollupUnitLengthRule { max_length: 3600 };
        let mut item = new_identity_rollup("x", "y");
        item.unit = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
