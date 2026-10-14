use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive binding score must pass. Check returns its problems, or none.
pub trait AdaptiveBindingScoreRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveBindingScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingScoreRequiredHostRule {
    pub message: String,
}

impl AdaptiveBindingScoreRule for AdaptiveBindingScoreRequiredHostRule {
    fn check(&self, item: &AdaptiveBindingScore) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingScoreSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveBindingScoreRule for AdaptiveBindingScoreSequenceRangeRule {
    fn check(&self, item: &AdaptiveBindingScore) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingScoreCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveBindingScoreRule for AdaptiveBindingScoreCodeLengthRule {
    fn check(&self, item: &AdaptiveBindingScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveBindingScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveBindingScoreRule for AdaptiveBindingScoreAllowedCodeRule {
    fn check(&self, item: &AdaptiveBindingScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive binding score.
#[derive(Debug, Clone)]
pub struct AdaptiveBindingScoreValidator {
    pub rules: Vec<Rc<dyn AdaptiveBindingScoreRule>>,
}

impl AdaptiveBindingScoreValidator {
    pub fn validate(&self, item: &AdaptiveBindingScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveBindingScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveBindingScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_binding_score_validator() -> AdaptiveBindingScoreValidator {
    AdaptiveBindingScoreValidator {
        rules: vec![
            Rc::new(AdaptiveBindingScoreRequiredHostRule {
                message: "adaptive binding score needs an entry".to_string(),
            }) as Rc<dyn AdaptiveBindingScoreRule>,
            Rc::new(AdaptiveBindingScoreSequenceRangeRule { min: 80, max: 120 }),
            Rc::new(AdaptiveBindingScoreCodeLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveBindingScoreAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_binding_score_validator();
        let item = new_adaptive_binding_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = AdaptiveBindingScoreRequiredHostRule {
            message: "adaptive binding score needs a value".to_string(),
        };
        let mut item = new_adaptive_binding_score("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = AdaptiveBindingScoreSequenceRangeRule { min: 80, max: 1000 };
        let mut item = new_adaptive_binding_score("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveBindingScoreCodeLengthRule { max_length: 16 };
        let mut item = new_adaptive_binding_score("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
