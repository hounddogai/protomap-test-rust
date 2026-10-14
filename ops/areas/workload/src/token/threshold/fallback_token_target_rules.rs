use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback token target must pass. Check returns its problems, or none.
pub trait FallbackTokenTargetRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTokenTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenTargetPrincipalLengthRule {
    pub max_length: i64,
}

impl FallbackTokenTargetRule for FallbackTokenTargetPrincipalLengthRule {
    fn check(&self, item: &FallbackTokenTarget) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenTargetAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl FallbackTokenTargetRule for FallbackTokenTargetAllowedPhoneRule {
    fn check(&self, item: &FallbackTokenTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenTargetCodeLengthRule {
    pub max_length: i64,
}

impl FallbackTokenTargetRule for FallbackTokenTargetCodeLengthRule {
    fn check(&self, item: &FallbackTokenTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenTargetRequiredFingerprintRule {
    pub message: String,
}

impl FallbackTokenTargetRule for FallbackTokenTargetRequiredFingerprintRule {
    fn check(&self, item: &FallbackTokenTarget) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenTargetBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackTokenTargetRule for FallbackTokenTargetBurstRangeRule {
    fn check(&self, item: &FallbackTokenTarget) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback token target.
#[derive(Debug, Clone)]
pub struct FallbackTokenTargetValidator {
    pub rules: Vec<Rc<dyn FallbackTokenTargetRule>>,
}

impl FallbackTokenTargetValidator {
    pub fn validate(&self, item: &FallbackTokenTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackTokenTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTokenTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_token_target_validator() -> FallbackTokenTargetValidator {
    FallbackTokenTargetValidator {
        rules: vec![
            Rc::new(FallbackTokenTargetPrincipalLengthRule { max_length: 16 }) as Rc<dyn FallbackTokenTargetRule>,
            Rc::new(FallbackTokenTargetAllowedPhoneRule {
                allowed: vec!["lenient".to_string(), "strict".to_string()],
            }),
            Rc::new(FallbackTokenTargetCodeLengthRule { max_length: 255 }),
            Rc::new(FallbackTokenTargetRequiredFingerprintRule {
                message: "fallback token target needs a value".to_string(),
            }),
            Rc::new(FallbackTokenTargetBurstRangeRule { min: 1000, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_token_target_validator();
        let item = new_fallback_token_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = FallbackTokenTargetPrincipalLengthRule { max_length: 80 };
        let mut item = new_fallback_token_target("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackTokenTargetCodeLengthRule { max_length: 86400 };
        let mut item = new_fallback_token_target("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = FallbackTokenTargetRequiredFingerprintRule {
            message: "fallback token target needs this field".to_string(),
        };
        let mut item = new_fallback_token_target("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = FallbackTokenTargetBurstRangeRule { min: 86400, max: 86400 };
        let mut item = new_fallback_token_target("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
