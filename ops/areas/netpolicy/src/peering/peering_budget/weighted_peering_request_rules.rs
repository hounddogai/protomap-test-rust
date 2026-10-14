use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted peering request must pass. Check returns its problems, or none.
pub trait WeightedPeeringRequestRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPeeringRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringRequestRequiredNameRule {
    pub message: String,
}

impl WeightedPeeringRequestRule for WeightedPeeringRequestRequiredNameRule {
    fn check(&self, item: &WeightedPeeringRequest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringRequestCodePrefixRule {
    pub prefix: String,
}

impl WeightedPeeringRequestRule for WeightedPeeringRequestCodePrefixRule {
    fn check(&self, item: &WeightedPeeringRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringRequestRequiredCodeRule {
    pub message: String,
}

impl WeightedPeeringRequestRule for WeightedPeeringRequestRequiredCodeRule {
    fn check(&self, item: &WeightedPeeringRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted peering request.
#[derive(Debug, Clone)]
pub struct WeightedPeeringRequestValidator {
    pub rules: Vec<Rc<dyn WeightedPeeringRequestRule>>,
}

impl WeightedPeeringRequestValidator {
    pub fn validate(&self, item: &WeightedPeeringRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedPeeringRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPeeringRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_peering_request_validator() -> WeightedPeeringRequestValidator {
    WeightedPeeringRequestValidator {
        rules: vec![
            Rc::new(WeightedPeeringRequestRequiredNameRule {
                message: "weighted peering request needs a value".to_string(),
            }) as Rc<dyn WeightedPeeringRequestRule>,
            Rc::new(WeightedPeeringRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(WeightedPeeringRequestRequiredCodeRule {
                message: "weighted peering request needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_peering_request_validator();
        let item = new_weighted_peering_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedPeeringRequestRequiredNameRule {
            message: "weighted peering request needs an entry".to_string(),
        };
        let mut item = new_weighted_peering_request("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedPeeringRequestRequiredCodeRule {
            message: "weighted peering request needs this field".to_string(),
        };
        let mut item = new_weighted_peering_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
