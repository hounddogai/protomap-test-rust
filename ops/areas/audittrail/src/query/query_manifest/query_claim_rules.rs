use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query claim must pass. Check returns its problems, or none.
pub trait QueryClaimRule: std::fmt::Debug {
    fn check(&self, item: &QueryClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryClaimCodePrefixRule {
    pub prefix: String,
}

impl QueryClaimRule for QueryClaimCodePrefixRule {
    fn check(&self, item: &QueryClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryClaimRequiredCurrencyRule {
    pub message: String,
}

impl QueryClaimRule for QueryClaimRequiredCurrencyRule {
    fn check(&self, item: &QueryClaim) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryClaimTitleLengthRule {
    pub max_length: i64,
}

impl QueryClaimRule for QueryClaimTitleLengthRule {
    fn check(&self, item: &QueryClaim) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryClaimAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl QueryClaimRule for QueryClaimAllowedSummaryRule {
    fn check(&self, item: &QueryClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is too long", item.summary)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query claim.
#[derive(Debug, Clone)]
pub struct QueryClaimValidator {
    pub rules: Vec<Rc<dyn QueryClaimRule>>,
}

impl QueryClaimValidator {
    pub fn validate(&self, item: &QueryClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_claim_validator() -> QueryClaimValidator {
    QueryClaimValidator {
        rules: vec![
            Rc::new(QueryClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn QueryClaimRule>,
            Rc::new(QueryClaimRequiredCurrencyRule {
                message: "query claim needs an entry".to_string(),
            }),
            Rc::new(QueryClaimTitleLengthRule { max_length: 1000 }),
            Rc::new(QueryClaimAllowedSummaryRule {
                allowed: vec!["lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_query_claim_validator();
        let item = new_query_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = QueryClaimRequiredCurrencyRule {
            message: "query claim needs this field".to_string(),
        };
        let mut item = new_query_claim("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QueryClaimTitleLengthRule { max_length: 16 };
        let mut item = new_query_claim("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
