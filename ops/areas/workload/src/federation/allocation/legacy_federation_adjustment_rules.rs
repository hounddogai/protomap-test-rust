use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy federation adjustment must pass. Check returns its problems, or none.
pub trait LegacyFederationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFederationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAdjustmentRequiredNameRule {
    pub message: String,
}

impl LegacyFederationAdjustmentRule for LegacyFederationAdjustmentRequiredNameRule {
    fn check(&self, item: &LegacyFederationAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl LegacyFederationAdjustmentRule for LegacyFederationAdjustmentNameLengthRule {
    fn check(&self, item: &LegacyFederationAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl LegacyFederationAdjustmentRule for LegacyFederationAdjustmentCodePrefixRule {
    fn check(&self, item: &LegacyFederationAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy federation adjustment.
#[derive(Debug, Clone)]
pub struct LegacyFederationAdjustmentValidator {
    pub rules: Vec<Rc<dyn LegacyFederationAdjustmentRule>>,
}

impl LegacyFederationAdjustmentValidator {
    pub fn validate(&self, item: &LegacyFederationAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyFederationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFederationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_federation_adjustment_validator() -> LegacyFederationAdjustmentValidator {
    LegacyFederationAdjustmentValidator {
        rules: vec![
            Rc::new(LegacyFederationAdjustmentRequiredNameRule {
                message: "legacy federation adjustment needs a value".to_string(),
            }) as Rc<dyn LegacyFederationAdjustmentRule>,
            Rc::new(LegacyFederationAdjustmentNameLengthRule { max_length: 32 }),
            Rc::new(LegacyFederationAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_federation_adjustment_validator();
        let item = new_legacy_federation_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyFederationAdjustmentRequiredNameRule {
            message: "legacy federation adjustment needs this field".to_string(),
        };
        let mut item = new_legacy_federation_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyFederationAdjustmentNameLengthRule { max_length: 16 };
        let mut item = new_legacy_federation_adjustment("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
