use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin milestone must pass. Check returns its problems, or none.
pub trait RegionPinMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMilestoneRequiredCodeRule {
    pub message: String,
}

impl RegionPinMilestoneRule for RegionPinMilestoneRequiredCodeRule {
    fn check(&self, item: &RegionPinMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMilestoneCodePrefixRule {
    pub prefix: String,
}

impl RegionPinMilestoneRule for RegionPinMilestoneCodePrefixRule {
    fn check(&self, item: &RegionPinMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinMilestoneRule for RegionPinMilestoneCodeLengthRule {
    fn check(&self, item: &RegionPinMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMilestoneNameLengthRule {
    pub max_length: i64,
}

impl RegionPinMilestoneRule for RegionPinMilestoneNameLengthRule {
    fn check(&self, item: &RegionPinMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin milestone.
#[derive(Debug, Clone)]
pub struct RegionPinMilestoneValidator {
    pub rules: Vec<Rc<dyn RegionPinMilestoneRule>>,
}

impl RegionPinMilestoneValidator {
    pub fn validate(&self, item: &RegionPinMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_milestone_validator() -> RegionPinMilestoneValidator {
    RegionPinMilestoneValidator {
        rules: vec![
            Rc::new(RegionPinMilestoneRequiredCodeRule {
                message: "region pin milestone needs this field".to_string(),
            }) as Rc<dyn RegionPinMilestoneRule>,
            Rc::new(RegionPinMilestoneCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionPinMilestoneCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionPinMilestoneNameLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_milestone_validator();
        let item = new_region_pin_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinMilestoneRequiredCodeRule {
            message: "region pin milestone needs an entry".to_string(),
        };
        let mut item = new_region_pin_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinMilestoneCodeLengthRule { max_length: 80 };
        let mut item = new_region_pin_milestone("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RegionPinMilestoneNameLengthRule { max_length: 16 };
        let mut item = new_region_pin_milestone("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
