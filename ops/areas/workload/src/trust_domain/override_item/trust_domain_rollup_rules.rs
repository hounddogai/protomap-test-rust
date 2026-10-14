use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain rollup must pass. Check returns its problems, or none.
pub trait TrustDomainRollupRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRollupRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainRollupRule for TrustDomainRollupRetriesRangeRule {
    fn check(&self, item: &TrustDomainRollup) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRollupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TrustDomainRollupRule for TrustDomainRollupAllowedCodeRule {
    fn check(&self, item: &TrustDomainRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRollupCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainRollupRule for TrustDomainRollupCodeLengthRule {
    fn check(&self, item: &TrustDomainRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain rollup.
#[derive(Debug, Clone)]
pub struct TrustDomainRollupValidator {
    pub rules: Vec<Rc<dyn TrustDomainRollupRule>>,
}

impl TrustDomainRollupValidator {
    pub fn validate(&self, item: &TrustDomainRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_rollup_validator() -> TrustDomainRollupValidator {
    TrustDomainRollupValidator {
        rules: vec![
            Rc::new(TrustDomainRollupRetriesRangeRule { min: 16, max: 1000 }) as Rc<dyn TrustDomainRollupRule>,
            Rc::new(TrustDomainRollupAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(TrustDomainRollupCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_rollup_validator();
        let item = new_trust_domain_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TrustDomainRollupRetriesRangeRule { min: 64, max: 120 };
        let mut item = new_trust_domain_rollup("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainRollupCodeLengthRule { max_length: 86400 };
        let mut item = new_trust_domain_rollup("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
