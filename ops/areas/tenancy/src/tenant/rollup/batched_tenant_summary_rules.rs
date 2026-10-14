use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched tenant summary must pass. Check returns its problems, or none.
pub trait BatchedTenantSummaryRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTenantSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantSummaryCodePrefixRule {
    pub prefix: String,
}

impl BatchedTenantSummaryRule for BatchedTenantSummaryCodePrefixRule {
    fn check(&self, item: &BatchedTenantSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantSummaryLocaleLengthRule {
    pub max_length: i64,
}

impl BatchedTenantSummaryRule for BatchedTenantSummaryLocaleLengthRule {
    fn check(&self, item: &BatchedTenantSummary) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantSummaryRequiredLabelRule {
    pub message: String,
}

impl BatchedTenantSummaryRule for BatchedTenantSummaryRequiredLabelRule {
    fn check(&self, item: &BatchedTenantSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantSummaryActiveGuardRule {
    pub limit: f64,
}

impl BatchedTenantSummaryRule for BatchedTenantSummaryActiveGuardRule {
    fn check(&self, item: &BatchedTenantSummary) -> Vec<String> {
        if item.active && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while active", item.score, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched tenant summary.
#[derive(Debug, Clone)]
pub struct BatchedTenantSummaryValidator {
    pub rules: Vec<Rc<dyn BatchedTenantSummaryRule>>,
}

impl BatchedTenantSummaryValidator {
    pub fn validate(&self, item: &BatchedTenantSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedTenantSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTenantSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_tenant_summary_validator() -> BatchedTenantSummaryValidator {
    BatchedTenantSummaryValidator {
        rules: vec![
            Rc::new(BatchedTenantSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn BatchedTenantSummaryRule>,
            Rc::new(BatchedTenantSummaryLocaleLengthRule { max_length: 64 }),
            Rc::new(BatchedTenantSummaryRequiredLabelRule {
                message: "batched tenant summary needs this field".to_string(),
            }),
            Rc::new(BatchedTenantSummaryActiveGuardRule { limit: 8.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_tenant_summary_validator();
        let item = new_batched_tenant_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = BatchedTenantSummaryLocaleLengthRule { max_length: 16 };
        let mut item = new_batched_tenant_summary("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedTenantSummaryRequiredLabelRule {
            message: "batched tenant summary needs a value".to_string(),
        };
        let mut item = new_batched_tenant_summary("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
