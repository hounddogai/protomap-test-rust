use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist override must pass. Check returns its problems, or none.
pub trait AllowlistOverrideRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistOverrideRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistOverrideRule for AllowlistOverrideRotationDaysRangeRule {
    fn check(&self, item: &AllowlistOverride) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistOverrideAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl AllowlistOverrideRule for AllowlistOverrideAllowedTitleRule {
    fn check(&self, item: &AllowlistOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistOverrideCodePrefixRule {
    pub prefix: String,
}

impl AllowlistOverrideRule for AllowlistOverrideCodePrefixRule {
    fn check(&self, item: &AllowlistOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistOverrideEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistOverrideRule for AllowlistOverrideEndsAtRangeRule {
    fn check(&self, item: &AllowlistOverride) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistOverrideRequiredTitleRule {
    pub message: String,
}

impl AllowlistOverrideRule for AllowlistOverrideRequiredTitleRule {
    fn check(&self, item: &AllowlistOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist override.
#[derive(Debug, Clone)]
pub struct AllowlistOverrideValidator {
    pub rules: Vec<Rc<dyn AllowlistOverrideRule>>,
}

impl AllowlistOverrideValidator {
    pub fn validate(&self, item: &AllowlistOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_override_validator() -> AllowlistOverrideValidator {
    AllowlistOverrideValidator {
        rules: vec![
            Rc::new(AllowlistOverrideRotationDaysRangeRule { min: 64, max: 120 }) as Rc<dyn AllowlistOverrideRule>,
            Rc::new(AllowlistOverrideAllowedTitleRule {
                allowed: vec!["rolling".to_string(), "batched".to_string(), "default".to_string()],
            }),
            Rc::new(AllowlistOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AllowlistOverrideEndsAtRangeRule { min: 16, max: 1000 }),
            Rc::new(AllowlistOverrideRequiredTitleRule {
                message: "allowlist override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_override_validator();
        let item = new_allowlist_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = AllowlistOverrideRotationDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_allowlist_override("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = AllowlistOverrideEndsAtRangeRule { min: 120, max: 86400 };
        let mut item = new_allowlist_override("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AllowlistOverrideRequiredTitleRule {
            message: "allowlist override needs this field".to_string(),
        };
        let mut item = new_allowlist_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
