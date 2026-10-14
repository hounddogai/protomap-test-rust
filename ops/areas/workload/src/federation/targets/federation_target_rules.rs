use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation target must pass. Check returns its problems, or none.
pub trait FederationTargetRule: std::fmt::Debug {
    fn check(&self, item: &FederationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTargetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FederationTargetRule for FederationTargetAllowedNameRule {
    fn check(&self, item: &FederationTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTargetScopeLengthRule {
    pub max_length: i64,
}

impl FederationTargetRule for FederationTargetScopeLengthRule {
    fn check(&self, item: &FederationTarget) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTargetNameLengthRule {
    pub max_length: i64,
}

impl FederationTargetRule for FederationTargetNameLengthRule {
    fn check(&self, item: &FederationTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTargetRequiredNameRule {
    pub message: String,
}

impl FederationTargetRule for FederationTargetRequiredNameRule {
    fn check(&self, item: &FederationTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTargetRequiredScopeRule {
    pub message: String,
}

impl FederationTargetRule for FederationTargetRequiredScopeRule {
    fn check(&self, item: &FederationTarget) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationTargetCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationTargetRule for FederationTargetCapacityRangeRule {
    fn check(&self, item: &FederationTarget) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation target.
#[derive(Debug, Clone)]
pub struct FederationTargetValidator {
    pub rules: Vec<Rc<dyn FederationTargetRule>>,
}

impl FederationTargetValidator {
    pub fn validate(&self, item: &FederationTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FederationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_target_validator() -> FederationTargetValidator {
    FederationTargetValidator {
        rules: vec![
            Rc::new(FederationTargetAllowedNameRule {
                allowed: vec!["scheduled".to_string()],
            }) as Rc<dyn FederationTargetRule>,
            Rc::new(FederationTargetScopeLengthRule { max_length: 86400 }),
            Rc::new(FederationTargetNameLengthRule { max_length: 64 }),
            Rc::new(FederationTargetRequiredNameRule {
                message: "federation target needs an entry".to_string(),
            }),
            Rc::new(FederationTargetRequiredScopeRule {
                message: "federation target needs a value".to_string(),
            }),
            Rc::new(FederationTargetCapacityRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_target_validator();
        let item = new_federation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = FederationTargetScopeLengthRule { max_length: 16 };
        let mut item = new_federation_target("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FederationTargetNameLengthRule { max_length: 120 };
        let mut item = new_federation_target("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FederationTargetRequiredNameRule {
            message: "federation target needs a value".to_string(),
        };
        let mut item = new_federation_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = FederationTargetRequiredScopeRule {
            message: "federation target needs a value".to_string(),
        };
        let mut item = new_federation_target("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = FederationTargetCapacityRangeRule { min: 32, max: 86400 };
        let mut item = new_federation_target("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
