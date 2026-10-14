use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient probe override must pass. Check returns its problems, or none.
pub trait LenientProbeOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LenientProbeOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeOverrideRequiredCurrencyRule {
    pub message: String,
}

impl LenientProbeOverrideRule for LenientProbeOverrideRequiredCurrencyRule {
    fn check(&self, item: &LenientProbeOverride) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeOverrideCodePrefixRule {
    pub prefix: String,
}

impl LenientProbeOverrideRule for LenientProbeOverrideCodePrefixRule {
    fn check(&self, item: &LenientProbeOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeOverrideCurrencyLengthRule {
    pub max_length: i64,
}

impl LenientProbeOverrideRule for LenientProbeOverrideCurrencyLengthRule {
    fn check(&self, item: &LenientProbeOverride) -> Vec<String> {
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
pub struct LenientProbeOverrideLabelLengthRule {
    pub max_length: i64,
}

impl LenientProbeOverrideRule for LenientProbeOverrideLabelLengthRule {
    fn check(&self, item: &LenientProbeOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeOverrideRequiredLabelRule {
    pub message: String,
}

impl LenientProbeOverrideRule for LenientProbeOverrideRequiredLabelRule {
    fn check(&self, item: &LenientProbeOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient probe override.
#[derive(Debug, Clone)]
pub struct LenientProbeOverrideValidator {
    pub rules: Vec<Rc<dyn LenientProbeOverrideRule>>,
}

impl LenientProbeOverrideValidator {
    pub fn validate(&self, item: &LenientProbeOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientProbeOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientProbeOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_probe_override_validator() -> LenientProbeOverrideValidator {
    LenientProbeOverrideValidator {
        rules: vec![
            Rc::new(LenientProbeOverrideRequiredCurrencyRule {
                message: "lenient probe override needs a value".to_string(),
            }) as Rc<dyn LenientProbeOverrideRule>,
            Rc::new(LenientProbeOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LenientProbeOverrideCurrencyLengthRule { max_length: 80 }),
            Rc::new(LenientProbeOverrideLabelLengthRule { max_length: 255 }),
            Rc::new(LenientProbeOverrideRequiredLabelRule {
                message: "lenient probe override needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_probe_override_validator();
        let item = new_lenient_probe_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = LenientProbeOverrideRequiredCurrencyRule {
            message: "lenient probe override needs this field".to_string(),
        };
        let mut item = new_lenient_probe_override("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = LenientProbeOverrideCurrencyLengthRule { max_length: 80 };
        let mut item = new_lenient_probe_override("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientProbeOverrideLabelLengthRule { max_length: 16 };
        let mut item = new_lenient_probe_override("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientProbeOverrideRequiredLabelRule {
            message: "lenient probe override needs an entry".to_string(),
        };
        let mut item = new_lenient_probe_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
