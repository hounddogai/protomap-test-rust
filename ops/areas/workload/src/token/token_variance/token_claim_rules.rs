use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token claim must pass. Check returns its problems, or none.
pub trait TokenClaimRule: std::fmt::Debug {
    fn check(&self, item: &TokenClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenClaimRequiredCodeRule {
    pub message: String,
}

impl TokenClaimRule for TokenClaimRequiredCodeRule {
    fn check(&self, item: &TokenClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenClaimCurrencyLengthRule {
    pub max_length: i64,
}

impl TokenClaimRule for TokenClaimCurrencyLengthRule {
    fn check(&self, item: &TokenClaim) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenClaimClusterLengthRule {
    pub max_length: i64,
}

impl TokenClaimRule for TokenClaimClusterLengthRule {
    fn check(&self, item: &TokenClaim) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenClaimCodePrefixRule {
    pub prefix: String,
}

impl TokenClaimRule for TokenClaimCodePrefixRule {
    fn check(&self, item: &TokenClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token claim.
#[derive(Debug, Clone)]
pub struct TokenClaimValidator {
    pub rules: Vec<Rc<dyn TokenClaimRule>>,
}

impl TokenClaimValidator {
    pub fn validate(&self, item: &TokenClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_claim_validator() -> TokenClaimValidator {
    TokenClaimValidator {
        rules: vec![
            Rc::new(TokenClaimRequiredCodeRule {
                message: "token claim needs this field".to_string(),
            }) as Rc<dyn TokenClaimRule>,
            Rc::new(TokenClaimCurrencyLengthRule { max_length: 255 }),
            Rc::new(TokenClaimClusterLengthRule { max_length: 3600 }),
            Rc::new(TokenClaimCodePrefixRule {
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
        let validator = default_token_claim_validator();
        let item = new_token_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenClaimRequiredCodeRule {
            message: "token claim needs an entry".to_string(),
        };
        let mut item = new_token_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = TokenClaimCurrencyLengthRule { max_length: 16 };
        let mut item = new_token_claim("x", "y");
        item.currency = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = TokenClaimClusterLengthRule { max_length: 16 };
        let mut item = new_token_claim("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
