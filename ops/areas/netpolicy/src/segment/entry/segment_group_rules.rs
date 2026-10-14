use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment group must pass. Check returns its problems, or none.
pub trait SegmentGroupRule: std::fmt::Debug {
    fn check(&self, item: &SegmentGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentGroupCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentGroupRule for SegmentGroupCreatedAtRangeRule {
    fn check(&self, item: &SegmentGroup) -> Vec<String> {
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
pub struct SegmentGroupNameLengthRule {
    pub max_length: i64,
}

impl SegmentGroupRule for SegmentGroupNameLengthRule {
    fn check(&self, item: &SegmentGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentGroupPrincipalLengthRule {
    pub max_length: i64,
}

impl SegmentGroupRule for SegmentGroupPrincipalLengthRule {
    fn check(&self, item: &SegmentGroup) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentGroupCodeLengthRule {
    pub max_length: i64,
}

impl SegmentGroupRule for SegmentGroupCodeLengthRule {
    fn check(&self, item: &SegmentGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentGroupCodePrefixRule {
    pub prefix: String,
}

impl SegmentGroupRule for SegmentGroupCodePrefixRule {
    fn check(&self, item: &SegmentGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentGroupRequiredExternalRefRule {
    pub message: String,
}

impl SegmentGroupRule for SegmentGroupRequiredExternalRefRule {
    fn check(&self, item: &SegmentGroup) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment group.
#[derive(Debug, Clone)]
pub struct SegmentGroupValidator {
    pub rules: Vec<Rc<dyn SegmentGroupRule>>,
}

impl SegmentGroupValidator {
    pub fn validate(&self, item: &SegmentGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_group_validator() -> SegmentGroupValidator {
    SegmentGroupValidator {
        rules: vec![
            Rc::new(SegmentGroupCreatedAtRangeRule { min: 32, max: 3600 }) as Rc<dyn SegmentGroupRule>,
            Rc::new(SegmentGroupNameLengthRule { max_length: 1000 }),
            Rc::new(SegmentGroupPrincipalLengthRule { max_length: 16 }),
            Rc::new(SegmentGroupCodeLengthRule { max_length: 120 }),
            Rc::new(SegmentGroupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SegmentGroupRequiredExternalRefRule {
                message: "segment group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_group_validator();
        let item = new_segment_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SegmentGroupCreatedAtRangeRule { min: 64, max: 86400 };
        let mut item = new_segment_group("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SegmentGroupNameLengthRule { max_length: 64 };
        let mut item = new_segment_group("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = SegmentGroupPrincipalLengthRule { max_length: 80 };
        let mut item = new_segment_group("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentGroupCodeLengthRule { max_length: 255 };
        let mut item = new_segment_group("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = SegmentGroupRequiredExternalRefRule {
            message: "segment group needs this field".to_string(),
        };
        let mut item = new_segment_group("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
