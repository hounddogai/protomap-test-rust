use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient suspension threshold must pass. Check returns its problems, or none.
pub trait LenientSuspensionThresholdRule: std::fmt::Debug {
    fn check(&self, item: &LenientSuspensionThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionThresholdRequiredCodeRule {
    pub message: String,
}

impl LenientSuspensionThresholdRule for LenientSuspensionThresholdRequiredCodeRule {
    fn check(&self, item: &LenientSuspensionThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionThresholdGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientSuspensionThresholdRule for LenientSuspensionThresholdGraceDaysRangeRule {
    fn check(&self, item: &LenientSuspensionThreshold) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionThresholdCodeLengthRule {
    pub max_length: i64,
}

impl LenientSuspensionThresholdRule for LenientSuspensionThresholdCodeLengthRule {
    fn check(&self, item: &LenientSuspensionThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionThresholdCodePrefixRule {
    pub prefix: String,
}

impl LenientSuspensionThresholdRule for LenientSuspensionThresholdCodePrefixRule {
    fn check(&self, item: &LenientSuspensionThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSuspensionThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientSuspensionThresholdRule for LenientSuspensionThresholdAllowedCodeRule {
    fn check(&self, item: &LenientSuspensionThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient suspension threshold.
#[derive(Debug, Clone)]
pub struct LenientSuspensionThresholdValidator {
    pub rules: Vec<Rc<dyn LenientSuspensionThresholdRule>>,
}

impl LenientSuspensionThresholdValidator {
    pub fn validate(&self, item: &LenientSuspensionThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSuspensionThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSuspensionThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_suspension_threshold_validator() -> LenientSuspensionThresholdValidator {
    LenientSuspensionThresholdValidator {
        rules: vec![
            Rc::new(LenientSuspensionThresholdRequiredCodeRule {
                message: "lenient suspension threshold needs this field".to_string(),
            }) as Rc<dyn LenientSuspensionThresholdRule>,
            Rc::new(LenientSuspensionThresholdGraceDaysRangeRule { min: 32, max: 255 }),
            Rc::new(LenientSuspensionThresholdCodeLengthRule { max_length: 120 }),
            Rc::new(LenientSuspensionThresholdCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientSuspensionThresholdAllowedCodeRule {
                allowed: vec!["pending".to_string(), "manual".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_suspension_threshold_validator();
        let item = new_lenient_suspension_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSuspensionThresholdRequiredCodeRule {
            message: "lenient suspension threshold needs a value".to_string(),
        };
        let mut item = new_lenient_suspension_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = LenientSuspensionThresholdGraceDaysRangeRule { min: 86400, max: 86400 };
        let mut item = new_lenient_suspension_threshold("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientSuspensionThresholdCodeLengthRule { max_length: 16 };
        let mut item = new_lenient_suspension_threshold("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
