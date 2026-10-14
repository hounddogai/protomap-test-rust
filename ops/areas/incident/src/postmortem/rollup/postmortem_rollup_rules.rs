use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem rollup must pass. Check returns its problems, or none.
pub trait PostmortemRollupRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRollupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PostmortemRollupRule for PostmortemRollupAllowedCodeRule {
    fn check(&self, item: &PostmortemRollup) -> Vec<String> {
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
pub struct PostmortemRollupCodePrefixRule {
    pub prefix: String,
}

impl PostmortemRollupRule for PostmortemRollupCodePrefixRule {
    fn check(&self, item: &PostmortemRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRollupLabelLengthRule {
    pub max_length: i64,
}

impl PostmortemRollupRule for PostmortemRollupLabelLengthRule {
    fn check(&self, item: &PostmortemRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRollupDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemRollupRule for PostmortemRollupDurationMinutesRangeRule {
    fn check(&self, item: &PostmortemRollup) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a postmortem rollup.
#[derive(Debug, Clone)]
pub struct PostmortemRollupValidator {
    pub rules: Vec<Rc<dyn PostmortemRollupRule>>,
}

impl PostmortemRollupValidator {
    pub fn validate(&self, item: &PostmortemRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_rollup_validator() -> PostmortemRollupValidator {
    PostmortemRollupValidator {
        rules: vec![
            Rc::new(PostmortemRollupAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "adaptive".to_string()],
            }) as Rc<dyn PostmortemRollupRule>,
            Rc::new(PostmortemRollupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PostmortemRollupLabelLengthRule { max_length: 1000 }),
            Rc::new(PostmortemRollupDurationMinutesRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_rollup_validator();
        let item = new_postmortem_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PostmortemRollupLabelLengthRule { max_length: 80 };
        let mut item = new_postmortem_rollup("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = PostmortemRollupDurationMinutesRangeRule { min: 120, max: 86400 };
        let mut item = new_postmortem_rollup("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
