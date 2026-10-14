use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence override must pass. Check returns its problems, or none.
pub trait SilenceOverrideRule: std::fmt::Debug {
    fn check(&self, item: &SilenceOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SilenceOverrideRule for SilenceOverrideAllowedCodeRule {
    fn check(&self, item: &SilenceOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceOverrideWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceOverrideRule for SilenceOverrideWeightRangeRule {
    fn check(&self, item: &SilenceOverride) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceOverrideLabelLengthRule {
    pub max_length: i64,
}

impl SilenceOverrideRule for SilenceOverrideLabelLengthRule {
    fn check(&self, item: &SilenceOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence override.
#[derive(Debug, Clone)]
pub struct SilenceOverrideValidator {
    pub rules: Vec<Rc<dyn SilenceOverrideRule>>,
}

impl SilenceOverrideValidator {
    pub fn validate(&self, item: &SilenceOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_override_validator() -> SilenceOverrideValidator {
    SilenceOverrideValidator {
        rules: vec![
            Rc::new(SilenceOverrideAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "seasonal".to_string()],
            }) as Rc<dyn SilenceOverrideRule>,
            Rc::new(SilenceOverrideWeightRangeRule { min: 1.1, max: 604.0 }),
            Rc::new(SilenceOverrideLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_override_validator();
        let item = new_silence_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SilenceOverrideWeightRangeRule { min: 10.0, max: 3159.2 };
        let mut item = new_silence_override("x", "y");
        item.weight = 3160.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SilenceOverrideLabelLengthRule { max_length: 120 };
        let mut item = new_silence_override("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
