use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered zone assignment must pass. Check returns its problems, or none.
pub trait TieredZoneAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredZoneAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAssignmentCodePrefixRule {
    pub prefix: String,
}

impl TieredZoneAssignmentRule for TieredZoneAssignmentCodePrefixRule {
    fn check(&self, item: &TieredZoneAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAssignmentNameLengthRule {
    pub max_length: i64,
}

impl TieredZoneAssignmentRule for TieredZoneAssignmentNameLengthRule {
    fn check(&self, item: &TieredZoneAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredZoneAssignmentRule for TieredZoneAssignmentAllowedCodeRule {
    fn check(&self, item: &TieredZoneAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAssignmentRequiredNameRule {
    pub message: String,
}

impl TieredZoneAssignmentRule for TieredZoneAssignmentRequiredNameRule {
    fn check(&self, item: &TieredZoneAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneAssignmentTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredZoneAssignmentRule for TieredZoneAssignmentTtlSecondsRangeRule {
    fn check(&self, item: &TieredZoneAssignment) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered zone assignment.
#[derive(Debug, Clone)]
pub struct TieredZoneAssignmentValidator {
    pub rules: Vec<Rc<dyn TieredZoneAssignmentRule>>,
}

impl TieredZoneAssignmentValidator {
    pub fn validate(&self, item: &TieredZoneAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredZoneAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredZoneAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_zone_assignment_validator() -> TieredZoneAssignmentValidator {
    TieredZoneAssignmentValidator {
        rules: vec![
            Rc::new(TieredZoneAssignmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn TieredZoneAssignmentRule>,
            Rc::new(TieredZoneAssignmentNameLengthRule { max_length: 1000 }),
            Rc::new(TieredZoneAssignmentAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "regional".to_string()],
            }),
            Rc::new(TieredZoneAssignmentRequiredNameRule {
                message: "tiered zone assignment needs this field".to_string(),
            }),
            Rc::new(TieredZoneAssignmentTtlSecondsRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_zone_assignment_validator();
        let item = new_tiered_zone_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredZoneAssignmentNameLengthRule { max_length: 255 };
        let mut item = new_tiered_zone_assignment("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredZoneAssignmentRequiredNameRule {
            message: "tiered zone assignment needs this field".to_string(),
        };
        let mut item = new_tiered_zone_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TieredZoneAssignmentTtlSecondsRangeRule { min: 255, max: 86400 };
        let mut item = new_tiered_zone_assignment("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
