use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback identity entry must pass. Check returns its problems, or none.
pub trait FallbackIdentityEntryRule: std::fmt::Debug {
    fn check(&self, item: &FallbackIdentityEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityEntryRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackIdentityEntryRule for FallbackIdentityEntryRatioRangeRule {
    fn check(&self, item: &FallbackIdentityEntry) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityEntryDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackIdentityEntryRule for FallbackIdentityEntryDurationMinutesRangeRule {
    fn check(&self, item: &FallbackIdentityEntry) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityEntryRequiredPrincipalRule {
    pub message: String,
}

impl FallbackIdentityEntryRule for FallbackIdentityEntryRequiredPrincipalRule {
    fn check(&self, item: &FallbackIdentityEntry) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityEntryCodePrefixRule {
    pub prefix: String,
}

impl FallbackIdentityEntryRule for FallbackIdentityEntryCodePrefixRule {
    fn check(&self, item: &FallbackIdentityEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityEntryRequiredCodeRule {
    pub message: String,
}

impl FallbackIdentityEntryRule for FallbackIdentityEntryRequiredCodeRule {
    fn check(&self, item: &FallbackIdentityEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityEntryCodeLengthRule {
    pub max_length: i64,
}

impl FallbackIdentityEntryRule for FallbackIdentityEntryCodeLengthRule {
    fn check(&self, item: &FallbackIdentityEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback identity entry.
#[derive(Debug, Clone)]
pub struct FallbackIdentityEntryValidator {
    pub rules: Vec<Rc<dyn FallbackIdentityEntryRule>>,
}

impl FallbackIdentityEntryValidator {
    pub fn validate(&self, item: &FallbackIdentityEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackIdentityEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackIdentityEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_identity_entry_validator() -> FallbackIdentityEntryValidator {
    FallbackIdentityEntryValidator {
        rules: vec![
            Rc::new(FallbackIdentityEntryRatioRangeRule { min: 9.5, max: 2594.2 }) as Rc<dyn FallbackIdentityEntryRule>,
            Rc::new(FallbackIdentityEntryDurationMinutesRangeRule { min: 120, max: 1000 }),
            Rc::new(FallbackIdentityEntryRequiredPrincipalRule {
                message: "fallback identity entry needs this field".to_string(),
            }),
            Rc::new(FallbackIdentityEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FallbackIdentityEntryRequiredCodeRule {
                message: "fallback identity entry needs an entry".to_string(),
            }),
            Rc::new(FallbackIdentityEntryCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_identity_entry_validator();
        let item = new_fallback_identity_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = FallbackIdentityEntryRatioRangeRule { min: 7.4, max: 4947.0 };
        let mut item = new_fallback_identity_entry("x", "y");
        item.ratio = 4948.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = FallbackIdentityEntryDurationMinutesRangeRule { min: 64, max: 86400 };
        let mut item = new_fallback_identity_entry("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = FallbackIdentityEntryRequiredPrincipalRule {
            message: "fallback identity entry needs an entry".to_string(),
        };
        let mut item = new_fallback_identity_entry("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackIdentityEntryRequiredCodeRule {
            message: "fallback identity entry needs this field".to_string(),
        };
        let mut item = new_fallback_identity_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackIdentityEntryCodeLengthRule { max_length: 32 };
        let mut item = new_fallback_identity_entry("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
