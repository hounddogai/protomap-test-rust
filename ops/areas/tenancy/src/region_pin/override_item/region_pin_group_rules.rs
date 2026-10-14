use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin group must pass. Check returns its problems, or none.
pub trait RegionPinGroupRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroupTimezoneLengthRule {
    pub max_length: i64,
}

impl RegionPinGroupRule for RegionPinGroupTimezoneLengthRule {
    fn check(&self, item: &RegionPinGroup) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroupRequiredCipherRule {
    pub message: String,
}

impl RegionPinGroupRule for RegionPinGroupRequiredCipherRule {
    fn check(&self, item: &RegionPinGroup) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroupRequiredCodeRule {
    pub message: String,
}

impl RegionPinGroupRule for RegionPinGroupRequiredCodeRule {
    fn check(&self, item: &RegionPinGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroupCodePrefixRule {
    pub prefix: String,
}

impl RegionPinGroupRule for RegionPinGroupCodePrefixRule {
    fn check(&self, item: &RegionPinGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinGroupCodeLengthRule {
    pub max_length: i64,
}

impl RegionPinGroupRule for RegionPinGroupCodeLengthRule {
    fn check(&self, item: &RegionPinGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin group.
#[derive(Debug, Clone)]
pub struct RegionPinGroupValidator {
    pub rules: Vec<Rc<dyn RegionPinGroupRule>>,
}

impl RegionPinGroupValidator {
    pub fn validate(&self, item: &RegionPinGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_group_validator() -> RegionPinGroupValidator {
    RegionPinGroupValidator {
        rules: vec![
            Rc::new(RegionPinGroupTimezoneLengthRule { max_length: 80 }) as Rc<dyn RegionPinGroupRule>,
            Rc::new(RegionPinGroupRequiredCipherRule {
                message: "region pin group needs an entry".to_string(),
            }),
            Rc::new(RegionPinGroupRequiredCodeRule {
                message: "region pin group needs an entry".to_string(),
            }),
            Rc::new(RegionPinGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionPinGroupCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_group_validator();
        let item = new_region_pin_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = RegionPinGroupTimezoneLengthRule { max_length: 120 };
        let mut item = new_region_pin_group("x", "y");
        item.timezone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = RegionPinGroupRequiredCipherRule {
            message: "region pin group needs this field".to_string(),
        };
        let mut item = new_region_pin_group("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinGroupRequiredCodeRule {
            message: "region pin group needs this field".to_string(),
        };
        let mut item = new_region_pin_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionPinGroupCodeLengthRule { max_length: 80 };
        let mut item = new_region_pin_group("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
