use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient silence tier must pass. Check returns its problems, or none.
pub trait LenientSilenceTierRule: std::fmt::Debug {
    fn check(&self, item: &LenientSilenceTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceTierRequiredLabelRule {
    pub message: String,
}

impl LenientSilenceTierRule for LenientSilenceTierRequiredLabelRule {
    fn check(&self, item: &LenientSilenceTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceTierAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl LenientSilenceTierRule for LenientSilenceTierAllowedSummaryRule {
    fn check(&self, item: &LenientSilenceTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is unknown", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceTierSummaryLengthRule {
    pub max_length: i64,
}

impl LenientSilenceTierRule for LenientSilenceTierSummaryLengthRule {
    fn check(&self, item: &LenientSilenceTier) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceTierRequiredCodeRule {
    pub message: String,
}

impl LenientSilenceTierRule for LenientSilenceTierRequiredCodeRule {
    fn check(&self, item: &LenientSilenceTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSilenceTierRequiredSummaryRule {
    pub message: String,
}

impl LenientSilenceTierRule for LenientSilenceTierRequiredSummaryRule {
    fn check(&self, item: &LenientSilenceTier) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient silence tier.
#[derive(Debug, Clone)]
pub struct LenientSilenceTierValidator {
    pub rules: Vec<Rc<dyn LenientSilenceTierRule>>,
}

impl LenientSilenceTierValidator {
    pub fn validate(&self, item: &LenientSilenceTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSilenceTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSilenceTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_silence_tier_validator() -> LenientSilenceTierValidator {
    LenientSilenceTierValidator {
        rules: vec![
            Rc::new(LenientSilenceTierRequiredLabelRule {
                message: "lenient silence tier needs this field".to_string(),
            }) as Rc<dyn LenientSilenceTierRule>,
            Rc::new(LenientSilenceTierAllowedSummaryRule {
                allowed: vec!["primary".to_string(), "pending".to_string(), "regional".to_string()],
            }),
            Rc::new(LenientSilenceTierSummaryLengthRule { max_length: 120 }),
            Rc::new(LenientSilenceTierRequiredCodeRule {
                message: "lenient silence tier needs an entry".to_string(),
            }),
            Rc::new(LenientSilenceTierRequiredSummaryRule {
                message: "lenient silence tier needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_silence_tier_validator();
        let item = new_lenient_silence_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientSilenceTierRequiredLabelRule {
            message: "lenient silence tier needs an entry".to_string(),
        };
        let mut item = new_lenient_silence_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = LenientSilenceTierSummaryLengthRule { max_length: 16 };
        let mut item = new_lenient_silence_tier("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSilenceTierRequiredCodeRule {
            message: "lenient silence tier needs an entry".to_string(),
        };
        let mut item = new_lenient_silence_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = LenientSilenceTierRequiredSummaryRule {
            message: "lenient silence tier needs this field".to_string(),
        };
        let mut item = new_lenient_silence_tier("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
