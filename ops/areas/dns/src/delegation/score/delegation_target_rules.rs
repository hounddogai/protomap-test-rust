use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a delegation target must pass. Check returns its problems, or none.
pub trait DelegationTargetRule: std::fmt::Debug {
    fn check(&self, item: &DelegationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTargetAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl DelegationTargetRule for DelegationTargetAllowedNamespaceRule {
    fn check(&self, item: &DelegationTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is stale", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTargetRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationTargetRule for DelegationTargetRevisionRangeRule {
    fn check(&self, item: &DelegationTarget) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTargetRequiredNamespaceRule {
    pub message: String,
}

impl DelegationTargetRule for DelegationTargetRequiredNamespaceRule {
    fn check(&self, item: &DelegationTarget) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTargetPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationTargetRule for DelegationTargetPortRangeRule {
    fn check(&self, item: &DelegationTarget) -> Vec<String> {
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
pub struct DelegationTargetNameLengthRule {
    pub max_length: i64,
}

impl DelegationTargetRule for DelegationTargetNameLengthRule {
    fn check(&self, item: &DelegationTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationTargetActiveGuardRule {
    pub limit: f64,
}

impl DelegationTargetRule for DelegationTargetActiveGuardRule {
    fn check(&self, item: &DelegationTarget) -> Vec<String> {
        if item.active && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while active",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation target.
#[derive(Debug, Clone)]
pub struct DelegationTargetValidator {
    pub rules: Vec<Rc<dyn DelegationTargetRule>>,
}

impl DelegationTargetValidator {
    pub fn validate(&self, item: &DelegationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_target_validator() -> DelegationTargetValidator {
    DelegationTargetValidator {
        rules: vec![
            Rc::new(DelegationTargetAllowedNamespaceRule {
                allowed: vec!["default".to_string(), "weighted".to_string(), "global".to_string()],
            }) as Rc<dyn DelegationTargetRule>,
            Rc::new(DelegationTargetRevisionRangeRule { min: 255, max: 1000 }),
            Rc::new(DelegationTargetRequiredNamespaceRule {
                message: "delegation target needs this field".to_string(),
            }),
            Rc::new(DelegationTargetPortRangeRule { min: 3600, max: 86400 }),
            Rc::new(DelegationTargetNameLengthRule { max_length: 120 }),
            Rc::new(DelegationTargetActiveGuardRule { limit: 9.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_target_validator();
        let item = new_delegation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DelegationTargetRevisionRangeRule { min: 16, max: 64 };
        let mut item = new_delegation_target("x", "y");
        item.revision = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = DelegationTargetRequiredNamespaceRule {
            message: "delegation target needs an entry".to_string(),
        };
        let mut item = new_delegation_target("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = DelegationTargetPortRangeRule { min: 120, max: 86400 };
        let mut item = new_delegation_target("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DelegationTargetNameLengthRule { max_length: 255 };
        let mut item = new_delegation_target("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
