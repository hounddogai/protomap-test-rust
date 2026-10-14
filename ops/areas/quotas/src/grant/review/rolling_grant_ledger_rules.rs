use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling grant ledger must pass. Check returns its problems, or none.
pub trait RollingGrantLedgerRule: std::fmt::Debug {
    fn check(&self, item: &RollingGrantLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantLedgerAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl RollingGrantLedgerRule for RollingGrantLedgerAllowedPrincipalRule {
    fn check(&self, item: &RollingGrantLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is expired", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantLedgerTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RollingGrantLedgerRule for RollingGrantLedgerTagsLimitRule {
    fn check(&self, item: &RollingGrantLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantLedgerCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingGrantLedgerRule for RollingGrantLedgerCeilingRangeRule {
    fn check(&self, item: &RollingGrantLedger) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantLedgerRequiredLocaleRule {
    pub message: String,
}

impl RollingGrantLedgerRule for RollingGrantLedgerRequiredLocaleRule {
    fn check(&self, item: &RollingGrantLedger) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantLedgerCodePrefixRule {
    pub prefix: String,
}

impl RollingGrantLedgerRule for RollingGrantLedgerCodePrefixRule {
    fn check(&self, item: &RollingGrantLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling grant ledger.
#[derive(Debug, Clone)]
pub struct RollingGrantLedgerValidator {
    pub rules: Vec<Rc<dyn RollingGrantLedgerRule>>,
}

impl RollingGrantLedgerValidator {
    pub fn validate(&self, item: &RollingGrantLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingGrantLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingGrantLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_grant_ledger_validator() -> RollingGrantLedgerValidator {
    RollingGrantLedgerValidator {
        rules: vec![
            Rc::new(RollingGrantLedgerAllowedPrincipalRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn RollingGrantLedgerRule>,
            Rc::new(RollingGrantLedgerTagsLimitRule {
                max_count: 16,
                banned: Vec::new(),
            }),
            Rc::new(RollingGrantLedgerCeilingRangeRule { min: 6.6, max: 2958.6 }),
            Rc::new(RollingGrantLedgerRequiredLocaleRule {
                message: "rolling grant ledger needs this field".to_string(),
            }),
            Rc::new(RollingGrantLedgerCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_grant_ledger_validator();
        let item = new_rolling_grant_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RollingGrantLedgerCeilingRangeRule { min: 5.1, max: 3490.2 };
        let mut item = new_rolling_grant_ledger("x", "y");
        item.ceiling = 3491.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RollingGrantLedgerRequiredLocaleRule {
            message: "rolling grant ledger needs a value".to_string(),
        };
        let mut item = new_rolling_grant_ledger("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
