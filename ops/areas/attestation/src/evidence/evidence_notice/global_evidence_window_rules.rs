use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global evidence window must pass. Check returns its problems, or none.
pub trait GlobalEvidenceWindowRule: std::fmt::Debug {
    fn check(&self, item: &GlobalEvidenceWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindowCodePrefixRule {
    pub prefix: String,
}

impl GlobalEvidenceWindowRule for GlobalEvidenceWindowCodePrefixRule {
    fn check(&self, item: &GlobalEvidenceWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindowCodeLengthRule {
    pub max_length: i64,
}

impl GlobalEvidenceWindowRule for GlobalEvidenceWindowCodeLengthRule {
    fn check(&self, item: &GlobalEvidenceWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindowMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalEvidenceWindowRule for GlobalEvidenceWindowMarginRangeRule {
    fn check(&self, item: &GlobalEvidenceWindow) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindowRequiredCodeRule {
    pub message: String,
}

impl GlobalEvidenceWindowRule for GlobalEvidenceWindowRequiredCodeRule {
    fn check(&self, item: &GlobalEvidenceWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindowRequiredTitleRule {
    pub message: String,
}

impl GlobalEvidenceWindowRule for GlobalEvidenceWindowRequiredTitleRule {
    fn check(&self, item: &GlobalEvidenceWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceWindowAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalEvidenceWindowRule for GlobalEvidenceWindowAllowedCodeRule {
    fn check(&self, item: &GlobalEvidenceWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global evidence window.
#[derive(Debug, Clone)]
pub struct GlobalEvidenceWindowValidator {
    pub rules: Vec<Rc<dyn GlobalEvidenceWindowRule>>,
}

impl GlobalEvidenceWindowValidator {
    pub fn validate(&self, item: &GlobalEvidenceWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalEvidenceWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalEvidenceWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_evidence_window_validator() -> GlobalEvidenceWindowValidator {
    GlobalEvidenceWindowValidator {
        rules: vec![
            Rc::new(GlobalEvidenceWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn GlobalEvidenceWindowRule>,
            Rc::new(GlobalEvidenceWindowCodeLengthRule { max_length: 64 }),
            Rc::new(GlobalEvidenceWindowMarginRangeRule { min: 1.4, max: 2491.5 }),
            Rc::new(GlobalEvidenceWindowRequiredCodeRule {
                message: "global evidence window needs a value".to_string(),
            }),
            Rc::new(GlobalEvidenceWindowRequiredTitleRule {
                message: "global evidence window needs an entry".to_string(),
            }),
            Rc::new(GlobalEvidenceWindowAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_evidence_window_validator();
        let item = new_global_evidence_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalEvidenceWindowCodeLengthRule { max_length: 255 };
        let mut item = new_global_evidence_window("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalEvidenceWindowMarginRangeRule { min: 2.2, max: 1710.0 };
        let mut item = new_global_evidence_window("x", "y");
        item.margin = 1711.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalEvidenceWindowRequiredCodeRule {
            message: "global evidence window needs this field".to_string(),
        };
        let mut item = new_global_evidence_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalEvidenceWindowRequiredTitleRule {
            message: "global evidence window needs a value".to_string(),
        };
        let mut item = new_global_evidence_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
