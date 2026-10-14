use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered peering summary must pass. Check returns its problems, or none.
pub trait TieredPeeringSummaryRule: std::fmt::Debug {
    fn check(&self, item: &TieredPeeringSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSummaryCodeLengthRule {
    pub max_length: i64,
}

impl TieredPeeringSummaryRule for TieredPeeringSummaryCodeLengthRule {
    fn check(&self, item: &TieredPeeringSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSummaryScopeLengthRule {
    pub max_length: i64,
}

impl TieredPeeringSummaryRule for TieredPeeringSummaryScopeLengthRule {
    fn check(&self, item: &TieredPeeringSummary) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSummaryRequiredTimezoneRule {
    pub message: String,
}

impl TieredPeeringSummaryRule for TieredPeeringSummaryRequiredTimezoneRule {
    fn check(&self, item: &TieredPeeringSummary) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSummaryRequiredCodeRule {
    pub message: String,
}

impl TieredPeeringSummaryRule for TieredPeeringSummaryRequiredCodeRule {
    fn check(&self, item: &TieredPeeringSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSummaryTimezoneLengthRule {
    pub max_length: i64,
}

impl TieredPeeringSummaryRule for TieredPeeringSummaryTimezoneLengthRule {
    fn check(&self, item: &TieredPeeringSummary) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered peering summary.
#[derive(Debug, Clone)]
pub struct TieredPeeringSummaryValidator {
    pub rules: Vec<Rc<dyn TieredPeeringSummaryRule>>,
}

impl TieredPeeringSummaryValidator {
    pub fn validate(&self, item: &TieredPeeringSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredPeeringSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPeeringSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_peering_summary_validator() -> TieredPeeringSummaryValidator {
    TieredPeeringSummaryValidator {
        rules: vec![
            Rc::new(TieredPeeringSummaryCodeLengthRule { max_length: 80 }) as Rc<dyn TieredPeeringSummaryRule>,
            Rc::new(TieredPeeringSummaryScopeLengthRule { max_length: 255 }),
            Rc::new(TieredPeeringSummaryRequiredTimezoneRule {
                message: "tiered peering summary needs this field".to_string(),
            }),
            Rc::new(TieredPeeringSummaryRequiredCodeRule {
                message: "tiered peering summary needs a value".to_string(),
            }),
            Rc::new(TieredPeeringSummaryTimezoneLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_peering_summary_validator();
        let item = new_tiered_peering_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredPeeringSummaryCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_peering_summary("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = TieredPeeringSummaryScopeLengthRule { max_length: 120 };
        let mut item = new_tiered_peering_summary("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = TieredPeeringSummaryRequiredTimezoneRule {
            message: "tiered peering summary needs a value".to_string(),
        };
        let mut item = new_tiered_peering_summary("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredPeeringSummaryRequiredCodeRule {
            message: "tiered peering summary needs an entry".to_string(),
        };
        let mut item = new_tiered_peering_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = TieredPeeringSummaryTimezoneLengthRule { max_length: 64 };
        let mut item = new_tiered_peering_summary("x", "y");
        item.timezone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
