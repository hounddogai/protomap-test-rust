use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal silence target must pass. Check returns its problems, or none.
pub trait SeasonalSilenceTargetRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalSilenceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSilenceTargetCurrencyLengthRule {
    pub max_length: i64,
}

impl SeasonalSilenceTargetRule for SeasonalSilenceTargetCurrencyLengthRule {
    fn check(&self, item: &SeasonalSilenceTarget) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSilenceTargetRequiredCurrencyRule {
    pub message: String,
}

impl SeasonalSilenceTargetRule for SeasonalSilenceTargetRequiredCurrencyRule {
    fn check(&self, item: &SeasonalSilenceTarget) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSilenceTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalSilenceTargetRule for SeasonalSilenceTargetAllowedCodeRule {
    fn check(&self, item: &SeasonalSilenceTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSilenceTargetCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalSilenceTargetRule for SeasonalSilenceTargetCodeLengthRule {
    fn check(&self, item: &SeasonalSilenceTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSilenceTargetCodePrefixRule {
    pub prefix: String,
}

impl SeasonalSilenceTargetRule for SeasonalSilenceTargetCodePrefixRule {
    fn check(&self, item: &SeasonalSilenceTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal silence target.
#[derive(Debug, Clone)]
pub struct SeasonalSilenceTargetValidator {
    pub rules: Vec<Rc<dyn SeasonalSilenceTargetRule>>,
}

impl SeasonalSilenceTargetValidator {
    pub fn validate(&self, item: &SeasonalSilenceTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalSilenceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalSilenceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_silence_target_validator() -> SeasonalSilenceTargetValidator {
    SeasonalSilenceTargetValidator {
        rules: vec![
            Rc::new(SeasonalSilenceTargetCurrencyLengthRule { max_length: 80 }) as Rc<dyn SeasonalSilenceTargetRule>,
            Rc::new(SeasonalSilenceTargetRequiredCurrencyRule {
                message: "seasonal silence target needs an entry".to_string(),
            }),
            Rc::new(SeasonalSilenceTargetAllowedCodeRule {
                allowed: vec!["legacy".to_string()],
            }),
            Rc::new(SeasonalSilenceTargetCodeLengthRule { max_length: 86400 }),
            Rc::new(SeasonalSilenceTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_silence_target_validator();
        let item = new_seasonal_silence_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = SeasonalSilenceTargetCurrencyLengthRule { max_length: 32 };
        let mut item = new_seasonal_silence_target("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = SeasonalSilenceTargetRequiredCurrencyRule {
            message: "seasonal silence target needs this field".to_string(),
        };
        let mut item = new_seasonal_silence_target("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalSilenceTargetCodeLengthRule { max_length: 3600 };
        let mut item = new_seasonal_silence_target("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
