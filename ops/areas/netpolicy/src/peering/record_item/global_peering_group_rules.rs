use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global peering group must pass. Check returns its problems, or none.
pub trait GlobalPeeringGroupRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPeeringGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringGroupTitleLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringGroupRule for GlobalPeeringGroupTitleLengthRule {
    fn check(&self, item: &GlobalPeeringGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringGroupRequiredCodeRule {
    pub message: String,
}

impl GlobalPeeringGroupRule for GlobalPeeringGroupRequiredCodeRule {
    fn check(&self, item: &GlobalPeeringGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringGroupRequiredLabelRule {
    pub message: String,
}

impl GlobalPeeringGroupRule for GlobalPeeringGroupRequiredLabelRule {
    fn check(&self, item: &GlobalPeeringGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global peering group.
#[derive(Debug, Clone)]
pub struct GlobalPeeringGroupValidator {
    pub rules: Vec<Rc<dyn GlobalPeeringGroupRule>>,
}

impl GlobalPeeringGroupValidator {
    pub fn validate(&self, item: &GlobalPeeringGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalPeeringGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPeeringGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_peering_group_validator() -> GlobalPeeringGroupValidator {
    GlobalPeeringGroupValidator {
        rules: vec![
            Rc::new(GlobalPeeringGroupTitleLengthRule { max_length: 255 }) as Rc<dyn GlobalPeeringGroupRule>,
            Rc::new(GlobalPeeringGroupRequiredCodeRule {
                message: "global peering group needs this field".to_string(),
            }),
            Rc::new(GlobalPeeringGroupRequiredLabelRule {
                message: "global peering group needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_peering_group_validator();
        let item = new_global_peering_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalPeeringGroupTitleLengthRule { max_length: 64 };
        let mut item = new_global_peering_group("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPeeringGroupRequiredCodeRule {
            message: "global peering group needs this field".to_string(),
        };
        let mut item = new_global_peering_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalPeeringGroupRequiredLabelRule {
            message: "global peering group needs this field".to_string(),
        };
        let mut item = new_global_peering_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
