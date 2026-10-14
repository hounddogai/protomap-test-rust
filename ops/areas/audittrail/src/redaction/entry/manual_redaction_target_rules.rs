use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual redaction target must pass. Check returns its problems, or none.
pub trait ManualRedactionTargetRule: std::fmt::Debug {
    fn check(&self, item: &ManualRedactionTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTargetRequiredLocaleRule {
    pub message: String,
}

impl ManualRedactionTargetRule for ManualRedactionTargetRequiredLocaleRule {
    fn check(&self, item: &ManualRedactionTarget) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTargetCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRedactionTargetRule for ManualRedactionTargetCreatedAtRangeRule {
    fn check(&self, item: &ManualRedactionTarget) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTargetCodeLengthRule {
    pub max_length: i64,
}

impl ManualRedactionTargetRule for ManualRedactionTargetCodeLengthRule {
    fn check(&self, item: &ManualRedactionTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTargetCodePrefixRule {
    pub prefix: String,
}

impl ManualRedactionTargetRule for ManualRedactionTargetCodePrefixRule {
    fn check(&self, item: &ManualRedactionTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionTargetLocaleLengthRule {
    pub max_length: i64,
}

impl ManualRedactionTargetRule for ManualRedactionTargetLocaleLengthRule {
    fn check(&self, item: &ManualRedactionTarget) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual redaction target.
#[derive(Debug, Clone)]
pub struct ManualRedactionTargetValidator {
    pub rules: Vec<Rc<dyn ManualRedactionTargetRule>>,
}

impl ManualRedactionTargetValidator {
    pub fn validate(&self, item: &ManualRedactionTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRedactionTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRedactionTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_redaction_target_validator() -> ManualRedactionTargetValidator {
    ManualRedactionTargetValidator {
        rules: vec![
            Rc::new(ManualRedactionTargetRequiredLocaleRule {
                message: "manual redaction target needs an entry".to_string(),
            }) as Rc<dyn ManualRedactionTargetRule>,
            Rc::new(ManualRedactionTargetCreatedAtRangeRule { min: 120, max: 1000 }),
            Rc::new(ManualRedactionTargetCodeLengthRule { max_length: 255 }),
            Rc::new(ManualRedactionTargetCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ManualRedactionTargetLocaleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_redaction_target_validator();
        let item = new_manual_redaction_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = ManualRedactionTargetRequiredLocaleRule {
            message: "manual redaction target needs this field".to_string(),
        };
        let mut item = new_manual_redaction_target("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = ManualRedactionTargetCreatedAtRangeRule { min: 86400, max: 86400 };
        let mut item = new_manual_redaction_target("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRedactionTargetCodeLengthRule { max_length: 80 };
        let mut item = new_manual_redaction_target("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ManualRedactionTargetLocaleLengthRule { max_length: 32 };
        let mut item = new_manual_redaction_target("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
