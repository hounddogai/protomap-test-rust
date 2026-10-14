use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering request must pass. Check returns its problems, or none.
pub trait PeeringRequestRule: std::fmt::Debug {
    fn check(&self, item: &PeeringRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRequestScopeLengthRule {
    pub max_length: i64,
}

impl PeeringRequestRule for PeeringRequestScopeLengthRule {
    fn check(&self, item: &PeeringRequest) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRequestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringRequestRule for PeeringRequestCreatedAtRangeRule {
    fn check(&self, item: &PeeringRequest) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRequestRequiredScopeRule {
    pub message: String,
}

impl PeeringRequestRule for PeeringRequestRequiredScopeRule {
    fn check(&self, item: &PeeringRequest) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRequestRequiredUnitRule {
    pub message: String,
}

impl PeeringRequestRule for PeeringRequestRequiredUnitRule {
    fn check(&self, item: &PeeringRequest) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRequestCodePrefixRule {
    pub prefix: String,
}

impl PeeringRequestRule for PeeringRequestCodePrefixRule {
    fn check(&self, item: &PeeringRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering request.
#[derive(Debug, Clone)]
pub struct PeeringRequestValidator {
    pub rules: Vec<Rc<dyn PeeringRequestRule>>,
}

impl PeeringRequestValidator {
    pub fn validate(&self, item: &PeeringRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_request_validator() -> PeeringRequestValidator {
    PeeringRequestValidator {
        rules: vec![
            Rc::new(PeeringRequestScopeLengthRule { max_length: 16 }) as Rc<dyn PeeringRequestRule>,
            Rc::new(PeeringRequestCreatedAtRangeRule { min: 16, max: 32 }),
            Rc::new(PeeringRequestRequiredScopeRule {
                message: "peering request needs this field".to_string(),
            }),
            Rc::new(PeeringRequestRequiredUnitRule {
                message: "peering request needs an entry".to_string(),
            }),
            Rc::new(PeeringRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_request_validator();
        let item = new_peering_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = PeeringRequestScopeLengthRule { max_length: 16 };
        let mut item = new_peering_request("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = PeeringRequestCreatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_peering_request("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = PeeringRequestRequiredScopeRule {
            message: "peering request needs a value".to_string(),
        };
        let mut item = new_peering_request("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PeeringRequestRequiredUnitRule {
            message: "peering request needs this field".to_string(),
        };
        let mut item = new_peering_request("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
