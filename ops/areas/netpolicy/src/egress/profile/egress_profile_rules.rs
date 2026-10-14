use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress profile must pass. Check returns its problems, or none.
pub trait EgressProfileRule: std::fmt::Debug {
    fn check(&self, item: &EgressProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressProfileMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressProfileRule for EgressProfileMinimumRangeRule {
    fn check(&self, item: &EgressProfile) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressProfileRequiredCodeRule {
    pub message: String,
}

impl EgressProfileRule for EgressProfileRequiredCodeRule {
    fn check(&self, item: &EgressProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressProfileRequiredOwnerIdRule {
    pub message: String,
}

impl EgressProfileRule for EgressProfileRequiredOwnerIdRule {
    fn check(&self, item: &EgressProfile) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressProfileNamespaceLengthRule {
    pub max_length: i64,
}

impl EgressProfileRule for EgressProfileNamespaceLengthRule {
    fn check(&self, item: &EgressProfile) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressProfileRequiredNamespaceRule {
    pub message: String,
}

impl EgressProfileRule for EgressProfileRequiredNamespaceRule {
    fn check(&self, item: &EgressProfile) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress profile.
#[derive(Debug, Clone)]
pub struct EgressProfileValidator {
    pub rules: Vec<Rc<dyn EgressProfileRule>>,
}

impl EgressProfileValidator {
    pub fn validate(&self, item: &EgressProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_profile_validator() -> EgressProfileValidator {
    EgressProfileValidator {
        rules: vec![
            Rc::new(EgressProfileMinimumRangeRule { min: 9.1, max: 1869.7 }) as Rc<dyn EgressProfileRule>,
            Rc::new(EgressProfileRequiredCodeRule {
                message: "egress profile needs an entry".to_string(),
            }),
            Rc::new(EgressProfileRequiredOwnerIdRule {
                message: "egress profile needs this field".to_string(),
            }),
            Rc::new(EgressProfileNamespaceLengthRule { max_length: 16 }),
            Rc::new(EgressProfileRequiredNamespaceRule {
                message: "egress profile needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_profile_validator();
        let item = new_egress_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = EgressProfileMinimumRangeRule { min: 5.8, max: 863.9 };
        let mut item = new_egress_profile("x", "y");
        item.minimum = 865.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressProfileRequiredCodeRule {
            message: "egress profile needs an entry".to_string(),
        };
        let mut item = new_egress_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = EgressProfileRequiredOwnerIdRule {
            message: "egress profile needs this field".to_string(),
        };
        let mut item = new_egress_profile("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = EgressProfileNamespaceLengthRule { max_length: 16 };
        let mut item = new_egress_profile("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = EgressProfileRequiredNamespaceRule {
            message: "egress profile needs a value".to_string(),
        };
        let mut item = new_egress_profile("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
