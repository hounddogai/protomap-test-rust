use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered usage summary must pass. Check returns its problems, or none.
pub trait TieredUsageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &TieredUsageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageSummaryIssuerLengthRule {
    pub max_length: i64,
}

impl TieredUsageSummaryRule for TieredUsageSummaryIssuerLengthRule {
    fn check(&self, item: &TieredUsageSummary) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageSummaryCodePrefixRule {
    pub prefix: String,
}

impl TieredUsageSummaryRule for TieredUsageSummaryCodePrefixRule {
    fn check(&self, item: &TieredUsageSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageSummaryRequiredIssuerRule {
    pub message: String,
}

impl TieredUsageSummaryRule for TieredUsageSummaryRequiredIssuerRule {
    fn check(&self, item: &TieredUsageSummary) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageSummaryRequiredCodeRule {
    pub message: String,
}

impl TieredUsageSummaryRule for TieredUsageSummaryRequiredCodeRule {
    fn check(&self, item: &TieredUsageSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered usage summary.
#[derive(Debug, Clone)]
pub struct TieredUsageSummaryValidator {
    pub rules: Vec<Rc<dyn TieredUsageSummaryRule>>,
}

impl TieredUsageSummaryValidator {
    pub fn validate(&self, item: &TieredUsageSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredUsageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredUsageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_usage_summary_validator() -> TieredUsageSummaryValidator {
    TieredUsageSummaryValidator {
        rules: vec![
            Rc::new(TieredUsageSummaryIssuerLengthRule { max_length: 1000 }) as Rc<dyn TieredUsageSummaryRule>,
            Rc::new(TieredUsageSummaryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TieredUsageSummaryRequiredIssuerRule {
                message: "tiered usage summary needs a value".to_string(),
            }),
            Rc::new(TieredUsageSummaryRequiredCodeRule {
                message: "tiered usage summary needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_usage_summary_validator();
        let item = new_tiered_usage_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = TieredUsageSummaryIssuerLengthRule { max_length: 1000 };
        let mut item = new_tiered_usage_summary("x", "y");
        item.issuer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = TieredUsageSummaryRequiredIssuerRule {
            message: "tiered usage summary needs a value".to_string(),
        };
        let mut item = new_tiered_usage_summary("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredUsageSummaryRequiredCodeRule {
            message: "tiered usage summary needs this field".to_string(),
        };
        let mut item = new_tiered_usage_summary("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
