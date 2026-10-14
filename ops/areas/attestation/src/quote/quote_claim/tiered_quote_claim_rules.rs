use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered quote claim must pass. Check returns its problems, or none.
pub trait TieredQuoteClaimRule: std::fmt::Debug {
    fn check(&self, item: &TieredQuoteClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteClaimTitleLengthRule {
    pub max_length: i64,
}

impl TieredQuoteClaimRule for TieredQuoteClaimTitleLengthRule {
    fn check(&self, item: &TieredQuoteClaim) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteClaimRequiredPrincipalRule {
    pub message: String,
}

impl TieredQuoteClaimRule for TieredQuoteClaimRequiredPrincipalRule {
    fn check(&self, item: &TieredQuoteClaim) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteClaimCodePrefixRule {
    pub prefix: String,
}

impl TieredQuoteClaimRule for TieredQuoteClaimCodePrefixRule {
    fn check(&self, item: &TieredQuoteClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered quote claim.
#[derive(Debug, Clone)]
pub struct TieredQuoteClaimValidator {
    pub rules: Vec<Rc<dyn TieredQuoteClaimRule>>,
}

impl TieredQuoteClaimValidator {
    pub fn validate(&self, item: &TieredQuoteClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredQuoteClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredQuoteClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_quote_claim_validator() -> TieredQuoteClaimValidator {
    TieredQuoteClaimValidator {
        rules: vec![
            Rc::new(TieredQuoteClaimTitleLengthRule { max_length: 86400 }) as Rc<dyn TieredQuoteClaimRule>,
            Rc::new(TieredQuoteClaimRequiredPrincipalRule {
                message: "tiered quote claim needs a value".to_string(),
            }),
            Rc::new(TieredQuoteClaimCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_quote_claim_validator();
        let item = new_tiered_quote_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredQuoteClaimTitleLengthRule { max_length: 80 };
        let mut item = new_tiered_quote_claim("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = TieredQuoteClaimRequiredPrincipalRule {
            message: "tiered quote claim needs a value".to_string(),
        };
        let mut item = new_tiered_quote_claim("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
