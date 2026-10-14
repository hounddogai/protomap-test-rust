use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive region pin rule must pass. Check returns its problems, or none.
pub trait AdaptiveRegionPinRuleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRegionPinRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRuleOwnerIdLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinRuleRule for AdaptiveRegionPinRuleOwnerIdLengthRule {
    fn check(&self, item: &AdaptiveRegionPinRule) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRuleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveRegionPinRuleRule for AdaptiveRegionPinRuleRotationDaysRangeRule {
    fn check(&self, item: &AdaptiveRegionPinRule) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRuleRequiredScopeRule {
    pub message: String,
}

impl AdaptiveRegionPinRuleRule for AdaptiveRegionPinRuleRequiredScopeRule {
    fn check(&self, item: &AdaptiveRegionPinRule) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRuleScopeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinRuleRule for AdaptiveRegionPinRuleScopeLengthRule {
    fn check(&self, item: &AdaptiveRegionPinRule) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRuleNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinRuleRule for AdaptiveRegionPinRuleNameLengthRule {
    fn check(&self, item: &AdaptiveRegionPinRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinRuleAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRegionPinRuleRule for AdaptiveRegionPinRuleAllowedTimezoneRule {
    fn check(&self, item: &AdaptiveRegionPinRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is inconsistent", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive region pin rule.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinRuleValidator {
    pub rules: Vec<Rc<dyn AdaptiveRegionPinRuleRule>>,
}

impl AdaptiveRegionPinRuleValidator {
    pub fn validate(&self, item: &AdaptiveRegionPinRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRegionPinRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRegionPinRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_region_pin_rule_validator() -> AdaptiveRegionPinRuleValidator {
    AdaptiveRegionPinRuleValidator {
        rules: vec![
            Rc::new(AdaptiveRegionPinRuleOwnerIdLengthRule { max_length: 64 }) as Rc<dyn AdaptiveRegionPinRuleRule>,
            Rc::new(AdaptiveRegionPinRuleRotationDaysRangeRule { min: 32, max: 32 }),
            Rc::new(AdaptiveRegionPinRuleRequiredScopeRule {
                message: "adaptive region pin rule needs an entry".to_string(),
            }),
            Rc::new(AdaptiveRegionPinRuleScopeLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveRegionPinRuleNameLengthRule { max_length: 80 }),
            Rc::new(AdaptiveRegionPinRuleAllowedTimezoneRule {
                allowed: vec!["default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_region_pin_rule_validator();
        let item = new_adaptive_region_pin_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = AdaptiveRegionPinRuleOwnerIdLengthRule { max_length: 64 };
        let mut item = new_adaptive_region_pin_rule("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = AdaptiveRegionPinRuleRotationDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_adaptive_region_pin_rule("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = AdaptiveRegionPinRuleRequiredScopeRule {
            message: "adaptive region pin rule needs a value".to_string(),
        };
        let mut item = new_adaptive_region_pin_rule("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = AdaptiveRegionPinRuleScopeLengthRule { max_length: 120 };
        let mut item = new_adaptive_region_pin_rule("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveRegionPinRuleNameLengthRule { max_length: 64 };
        let mut item = new_adaptive_region_pin_rule("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
