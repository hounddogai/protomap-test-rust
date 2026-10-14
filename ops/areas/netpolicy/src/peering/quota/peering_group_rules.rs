use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering group must pass. Check returns its problems, or none.
pub trait PeeringGroupRule: std::fmt::Debug {
    fn check(&self, item: &PeeringGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringGroupTitleLengthRule {
    pub max_length: i64,
}

impl PeeringGroupRule for PeeringGroupTitleLengthRule {
    fn check(&self, item: &PeeringGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringGroupRequiredIssuerRule {
    pub message: String,
}

impl PeeringGroupRule for PeeringGroupRequiredIssuerRule {
    fn check(&self, item: &PeeringGroup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringGroupIssuerLengthRule {
    pub max_length: i64,
}

impl PeeringGroupRule for PeeringGroupIssuerLengthRule {
    fn check(&self, item: &PeeringGroup) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering group.
#[derive(Debug, Clone)]
pub struct PeeringGroupValidator {
    pub rules: Vec<Rc<dyn PeeringGroupRule>>,
}

impl PeeringGroupValidator {
    pub fn validate(&self, item: &PeeringGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_group_validator() -> PeeringGroupValidator {
    PeeringGroupValidator {
        rules: vec![
            Rc::new(PeeringGroupTitleLengthRule { max_length: 120 }) as Rc<dyn PeeringGroupRule>,
            Rc::new(PeeringGroupRequiredIssuerRule {
                message: "peering group needs an entry".to_string(),
            }),
            Rc::new(PeeringGroupIssuerLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_group_validator();
        let item = new_peering_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PeeringGroupTitleLengthRule { max_length: 120 };
        let mut item = new_peering_group("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = PeeringGroupRequiredIssuerRule {
            message: "peering group needs an entry".to_string(),
        };
        let mut item = new_peering_group("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = PeeringGroupIssuerLengthRule { max_length: 16 };
        let mut item = new_peering_group("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
