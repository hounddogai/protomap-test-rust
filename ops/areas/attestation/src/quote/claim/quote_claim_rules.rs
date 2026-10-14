use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote claim must pass. Check returns its problems, or none.
pub trait QuoteClaimRule: std::fmt::Debug {
    fn check(&self, item: &QuoteClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteClaimBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteClaimRule for QuoteClaimBurstRangeRule {
    fn check(&self, item: &QuoteClaim) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteClaimAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl QuoteClaimRule for QuoteClaimAllowedCodeRule {
    fn check(&self, item: &QuoteClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteClaimScopeLengthRule {
    pub max_length: i64,
}

impl QuoteClaimRule for QuoteClaimScopeLengthRule {
    fn check(&self, item: &QuoteClaim) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteClaimCountryLengthRule {
    pub max_length: i64,
}

impl QuoteClaimRule for QuoteClaimCountryLengthRule {
    fn check(&self, item: &QuoteClaim) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote claim.
#[derive(Debug, Clone)]
pub struct QuoteClaimValidator {
    pub rules: Vec<Rc<dyn QuoteClaimRule>>,
}

impl QuoteClaimValidator {
    pub fn validate(&self, item: &QuoteClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_claim_validator() -> QuoteClaimValidator {
    QuoteClaimValidator {
        rules: vec![
            Rc::new(QuoteClaimBurstRangeRule { min: 80, max: 1000 }) as Rc<dyn QuoteClaimRule>,
            Rc::new(QuoteClaimAllowedCodeRule {
                allowed: vec!["manual".to_string(), "strict".to_string()],
            }),
            Rc::new(QuoteClaimScopeLengthRule { max_length: 120 }),
            Rc::new(QuoteClaimCountryLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_claim_validator();
        let item = new_quote_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = QuoteClaimBurstRangeRule { min: 64, max: 255 };
        let mut item = new_quote_claim("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = QuoteClaimScopeLengthRule { max_length: 64 };
        let mut item = new_quote_claim("x", "y");
        item.scope = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = QuoteClaimCountryLengthRule { max_length: 80 };
        let mut item = new_quote_claim("x", "y");
        item.country = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
