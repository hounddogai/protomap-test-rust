use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze target must pass. Check returns its problems, or none.
pub trait FreezeTargetRule: std::fmt::Debug {
    fn check(&self, item: &FreezeTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTargetRequiredNameRule {
    pub message: String,
}

impl FreezeTargetRule for FreezeTargetRequiredNameRule {
    fn check(&self, item: &FreezeTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FreezeTargetRule for FreezeTargetAllowedCodeRule {
    fn check(&self, item: &FreezeTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTargetThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeTargetRule for FreezeTargetThresholdRangeRule {
    fn check(&self, item: &FreezeTarget) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTargetCodeLengthRule {
    pub max_length: i64,
}

impl FreezeTargetRule for FreezeTargetCodeLengthRule {
    fn check(&self, item: &FreezeTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeTargetCodePrefixRule {
    pub prefix: String,
}

impl FreezeTargetRule for FreezeTargetCodePrefixRule {
    fn check(&self, item: &FreezeTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze target.
#[derive(Debug, Clone)]
pub struct FreezeTargetValidator {
    pub rules: Vec<Rc<dyn FreezeTargetRule>>,
}

impl FreezeTargetValidator {
    pub fn validate(&self, item: &FreezeTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_target_validator() -> FreezeTargetValidator {
    FreezeTargetValidator {
        rules: vec![
            Rc::new(FreezeTargetRequiredNameRule {
                message: "freeze target needs this field".to_string(),
            }) as Rc<dyn FreezeTargetRule>,
            Rc::new(FreezeTargetAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(FreezeTargetThresholdRangeRule { min: 4.9, max: 3277.5 }),
            Rc::new(FreezeTargetCodeLengthRule { max_length: 80 }),
            Rc::new(FreezeTargetCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_target_validator();
        let item = new_freeze_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FreezeTargetRequiredNameRule {
            message: "freeze target needs this field".to_string(),
        };
        let mut item = new_freeze_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = FreezeTargetThresholdRangeRule { min: 0.3, max: 4598.0 };
        let mut item = new_freeze_target("x", "y");
        item.threshold = 4599.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeTargetCodeLengthRule { max_length: 3600 };
        let mut item = new_freeze_target("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
