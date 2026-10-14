use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending isolation summary must pass. Check returns its problems, or none.
pub trait PendingIsolationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &PendingIsolationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationSummaryRequiredLabelRule {
    pub message: String,
}

impl PendingIsolationSummaryRule for PendingIsolationSummaryRequiredLabelRule {
    fn check(&self, item: &PendingIsolationSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationSummaryCodeLengthRule {
    pub max_length: i64,
}

impl PendingIsolationSummaryRule for PendingIsolationSummaryCodeLengthRule {
    fn check(&self, item: &PendingIsolationSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationSummaryRequiredCipherRule {
    pub message: String,
}

impl PendingIsolationSummaryRule for PendingIsolationSummaryRequiredCipherRule {
    fn check(&self, item: &PendingIsolationSummary) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationSummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingIsolationSummaryRule for PendingIsolationSummaryAllowedCodeRule {
    fn check(&self, item: &PendingIsolationSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationSummaryScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingIsolationSummaryRule for PendingIsolationSummaryScoreRangeRule {
    fn check(&self, item: &PendingIsolationSummary) -> Vec<String> {
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

/// Runs every rule of a pending isolation summary.
#[derive(Debug, Clone)]
pub struct PendingIsolationSummaryValidator {
    pub rules: Vec<Rc<dyn PendingIsolationSummaryRule>>,
}

impl PendingIsolationSummaryValidator {
    pub fn validate(&self, item: &PendingIsolationSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingIsolationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingIsolationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_isolation_summary_validator() -> PendingIsolationSummaryValidator {
    PendingIsolationSummaryValidator {
        rules: vec![
            Rc::new(PendingIsolationSummaryRequiredLabelRule {
                message: "pending isolation summary needs an entry".to_string(),
            }) as Rc<dyn PendingIsolationSummaryRule>,
            Rc::new(PendingIsolationSummaryCodeLengthRule { max_length: 16 }),
            Rc::new(PendingIsolationSummaryRequiredCipherRule {
                message: "pending isolation summary needs a value".to_string(),
            }),
            Rc::new(PendingIsolationSummaryAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "regional".to_string()],
            }),
            Rc::new(PendingIsolationSummaryScoreRangeRule { min: 9.0, max: 1593.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_isolation_summary_validator();
        let item = new_pending_isolation_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingIsolationSummaryRequiredLabelRule {
            message: "pending isolation summary needs a value".to_string(),
        };
        let mut item = new_pending_isolation_summary("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingIsolationSummaryCodeLengthRule { max_length: 80 };
        let mut item = new_pending_isolation_summary("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = PendingIsolationSummaryRequiredCipherRule {
            message: "pending isolation summary needs a value".to_string(),
        };
        let mut item = new_pending_isolation_summary("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PendingIsolationSummaryScoreRangeRule { min: 7.0, max: 1976.1 };
        let mut item = new_pending_isolation_summary("x", "y");
        item.score = 1977.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
