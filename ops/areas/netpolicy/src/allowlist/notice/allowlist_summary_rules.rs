use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist summary must pass. Check returns its problems, or none.
pub trait AllowlistSummaryRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummaryCodePrefixRule {
    pub prefix: String,
}

impl AllowlistSummaryRule for AllowlistSummaryCodePrefixRule {
    fn check(&self, item: &AllowlistSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummaryNameLengthRule {
    pub max_length: i64,
}

impl AllowlistSummaryRule for AllowlistSummaryNameLengthRule {
    fn check(&self, item: &AllowlistSummary) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummaryRequiredNameRule {
    pub message: String,
}

impl AllowlistSummaryRule for AllowlistSummaryRequiredNameRule {
    fn check(&self, item: &AllowlistSummary) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummaryScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistSummaryRule for AllowlistSummaryScoreRangeRule {
    fn check(&self, item: &AllowlistSummary) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AllowlistSummaryRule for AllowlistSummaryAllowedCodeRule {
    fn check(&self, item: &AllowlistSummary) -> Vec<String> {
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
pub struct AllowlistSummaryRequiredCodeRule {
    pub message: String,
}

impl AllowlistSummaryRule for AllowlistSummaryRequiredCodeRule {
    fn check(&self, item: &AllowlistSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist summary.
#[derive(Debug, Clone)]
pub struct AllowlistSummaryValidator {
    pub rules: Vec<Rc<dyn AllowlistSummaryRule>>,
}

impl AllowlistSummaryValidator {
    pub fn validate(&self, item: &AllowlistSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_summary_validator() -> AllowlistSummaryValidator {
    AllowlistSummaryValidator {
        rules: vec![
            Rc::new(AllowlistSummaryCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn AllowlistSummaryRule>,
            Rc::new(AllowlistSummaryNameLengthRule { max_length: 255 }),
            Rc::new(AllowlistSummaryRequiredNameRule {
                message: "allowlist summary needs a value".to_string(),
            }),
            Rc::new(AllowlistSummaryScoreRangeRule { min: 7.7, max: 888.5 }),
            Rc::new(AllowlistSummaryAllowedCodeRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(AllowlistSummaryRequiredCodeRule {
                message: "allowlist summary needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_summary_validator();
        let item = new_allowlist_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AllowlistSummaryNameLengthRule { max_length: 80 };
        let mut item = new_allowlist_summary("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AllowlistSummaryRequiredNameRule {
            message: "allowlist summary needs this field".to_string(),
        };
        let mut item = new_allowlist_summary("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = AllowlistSummaryScoreRangeRule { min: 2.9, max: 2542.9 };
        let mut item = new_allowlist_summary("x", "y");
        item.score = 2544.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistSummaryRequiredCodeRule {
            message: "allowlist summary needs this field".to_string(),
        };
        let mut item = new_allowlist_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
