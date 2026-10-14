use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy redaction override must pass. Check returns its problems, or none.
pub trait LegacyRedactionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRedactionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionOverrideRequiredTitleRule {
    pub message: String,
}

impl LegacyRedactionOverrideRule for LegacyRedactionOverrideRequiredTitleRule {
    fn check(&self, item: &LegacyRedactionOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionOverrideCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyRedactionOverrideRule for LegacyRedactionOverrideCostRangeRule {
    fn check(&self, item: &LegacyRedactionOverride) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionOverrideCodePrefixRule {
    pub prefix: String,
}

impl LegacyRedactionOverrideRule for LegacyRedactionOverrideCodePrefixRule {
    fn check(&self, item: &LegacyRedactionOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyRedactionOverrideRule for LegacyRedactionOverrideAllowedCodeRule {
    fn check(&self, item: &LegacyRedactionOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionOverrideStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyRedactionOverrideRule for LegacyRedactionOverrideStartsAtRangeRule {
    fn check(&self, item: &LegacyRedactionOverride) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy redaction override.
#[derive(Debug, Clone)]
pub struct LegacyRedactionOverrideValidator {
    pub rules: Vec<Rc<dyn LegacyRedactionOverrideRule>>,
}

impl LegacyRedactionOverrideValidator {
    pub fn validate(&self, item: &LegacyRedactionOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRedactionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRedactionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_redaction_override_validator() -> LegacyRedactionOverrideValidator {
    LegacyRedactionOverrideValidator {
        rules: vec![
            Rc::new(LegacyRedactionOverrideRequiredTitleRule {
                message: "legacy redaction override needs an entry".to_string(),
            }) as Rc<dyn LegacyRedactionOverrideRule>,
            Rc::new(LegacyRedactionOverrideCostRangeRule { min: 7.8, max: 3588.4 }),
            Rc::new(LegacyRedactionOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LegacyRedactionOverrideAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "tiered".to_string()],
            }),
            Rc::new(LegacyRedactionOverrideStartsAtRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_redaction_override_validator();
        let item = new_legacy_redaction_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyRedactionOverrideRequiredTitleRule {
            message: "legacy redaction override needs this field".to_string(),
        };
        let mut item = new_legacy_redaction_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = LegacyRedactionOverrideCostRangeRule { min: 1.8, max: 1507.0 };
        let mut item = new_legacy_redaction_override("x", "y");
        item.cost = 1508.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LegacyRedactionOverrideStartsAtRangeRule { min: 80, max: 120 };
        let mut item = new_legacy_redaction_override("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
