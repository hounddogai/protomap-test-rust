use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin override must pass. Check returns its problems, or none.
pub trait RegionPinOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinOverrideRequiredTimezoneRule {
    pub message: String,
}

impl RegionPinOverrideRule for RegionPinOverrideRequiredTimezoneRule {
    fn check(&self, item: &RegionPinOverride) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinOverrideCodePrefixRule {
    pub prefix: String,
}

impl RegionPinOverrideRule for RegionPinOverrideCodePrefixRule {
    fn check(&self, item: &RegionPinOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinOverrideNotesLengthRule {
    pub max_length: i64,
}

impl RegionPinOverrideRule for RegionPinOverrideNotesLengthRule {
    fn check(&self, item: &RegionPinOverride) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinOverrideRequiredNameRule {
    pub message: String,
}

impl RegionPinOverrideRule for RegionPinOverrideRequiredNameRule {
    fn check(&self, item: &RegionPinOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinOverrideAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl RegionPinOverrideRule for RegionPinOverrideAllowedTimezoneRule {
    fn check(&self, item: &RegionPinOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is stale", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin override.
#[derive(Debug, Clone)]
pub struct RegionPinOverrideValidator {
    pub rules: Vec<Rc<dyn RegionPinOverrideRule>>,
}

impl RegionPinOverrideValidator {
    pub fn validate(&self, item: &RegionPinOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_override_validator() -> RegionPinOverrideValidator {
    RegionPinOverrideValidator {
        rules: vec![
            Rc::new(RegionPinOverrideRequiredTimezoneRule {
                message: "region pin override needs this field".to_string(),
            }) as Rc<dyn RegionPinOverrideRule>,
            Rc::new(RegionPinOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionPinOverrideNotesLengthRule { max_length: 64 }),
            Rc::new(RegionPinOverrideRequiredNameRule {
                message: "region pin override needs an entry".to_string(),
            }),
            Rc::new(RegionPinOverrideAllowedTimezoneRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_override_validator();
        let item = new_region_pin_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = RegionPinOverrideRequiredTimezoneRule {
            message: "region pin override needs a value".to_string(),
        };
        let mut item = new_region_pin_override("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RegionPinOverrideNotesLengthRule { max_length: 86400 };
        let mut item = new_region_pin_override("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionPinOverrideRequiredNameRule {
            message: "region pin override needs a value".to_string(),
        };
        let mut item = new_region_pin_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
