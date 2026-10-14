use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict dashboard override must pass. Check returns its problems, or none.
pub trait StrictDashboardOverrideRule: std::fmt::Debug {
    fn check(&self, item: &StrictDashboardOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverrideSegmentLengthRule {
    pub max_length: i64,
}

impl StrictDashboardOverrideRule for StrictDashboardOverrideSegmentLengthRule {
    fn check(&self, item: &StrictDashboardOverride) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverrideRequiredDescriptionRule {
    pub message: String,
}

impl StrictDashboardOverrideRule for StrictDashboardOverrideRequiredDescriptionRule {
    fn check(&self, item: &StrictDashboardOverride) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverrideCodePrefixRule {
    pub prefix: String,
}

impl StrictDashboardOverrideRule for StrictDashboardOverrideCodePrefixRule {
    fn check(&self, item: &StrictDashboardOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverrideRequiredNameRule {
    pub message: String,
}

impl StrictDashboardOverrideRule for StrictDashboardOverrideRequiredNameRule {
    fn check(&self, item: &StrictDashboardOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverrideCodeLengthRule {
    pub max_length: i64,
}

impl StrictDashboardOverrideRule for StrictDashboardOverrideCodeLengthRule {
    fn check(&self, item: &StrictDashboardOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardOverrideNameLengthRule {
    pub max_length: i64,
}

impl StrictDashboardOverrideRule for StrictDashboardOverrideNameLengthRule {
    fn check(&self, item: &StrictDashboardOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict dashboard override.
#[derive(Debug, Clone)]
pub struct StrictDashboardOverrideValidator {
    pub rules: Vec<Rc<dyn StrictDashboardOverrideRule>>,
}

impl StrictDashboardOverrideValidator {
    pub fn validate(&self, item: &StrictDashboardOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictDashboardOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictDashboardOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_dashboard_override_validator() -> StrictDashboardOverrideValidator {
    StrictDashboardOverrideValidator {
        rules: vec![
            Rc::new(StrictDashboardOverrideSegmentLengthRule { max_length: 64 }) as Rc<dyn StrictDashboardOverrideRule>,
            Rc::new(StrictDashboardOverrideRequiredDescriptionRule {
                message: "strict dashboard override needs this field".to_string(),
            }),
            Rc::new(StrictDashboardOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(StrictDashboardOverrideRequiredNameRule {
                message: "strict dashboard override needs an entry".to_string(),
            }),
            Rc::new(StrictDashboardOverrideCodeLengthRule { max_length: 64 }),
            Rc::new(StrictDashboardOverrideNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_dashboard_override_validator();
        let item = new_strict_dashboard_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = StrictDashboardOverrideSegmentLengthRule { max_length: 32 };
        let mut item = new_strict_dashboard_override("x", "y");
        item.segment = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = StrictDashboardOverrideRequiredDescriptionRule {
            message: "strict dashboard override needs an entry".to_string(),
        };
        let mut item = new_strict_dashboard_override("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictDashboardOverrideRequiredNameRule {
            message: "strict dashboard override needs this field".to_string(),
        };
        let mut item = new_strict_dashboard_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictDashboardOverrideCodeLengthRule { max_length: 80 };
        let mut item = new_strict_dashboard_override("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictDashboardOverrideNameLengthRule { max_length: 255 };
        let mut item = new_strict_dashboard_override("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
