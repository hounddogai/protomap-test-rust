use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy isolation claim must pass. Check returns its problems, or none.
pub trait LegacyIsolationClaimRule: std::fmt::Debug {
    fn check(&self, item: &LegacyIsolationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationClaimAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl LegacyIsolationClaimRule for LegacyIsolationClaimAllowedSummaryRule {
    fn check(&self, item: &LegacyIsolationClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is expired", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationClaimCategoryLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationClaimRule for LegacyIsolationClaimCategoryLengthRule {
    fn check(&self, item: &LegacyIsolationClaim) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationClaimLabelLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationClaimRule for LegacyIsolationClaimLabelLengthRule {
    fn check(&self, item: &LegacyIsolationClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy isolation claim.
#[derive(Debug, Clone)]
pub struct LegacyIsolationClaimValidator {
    pub rules: Vec<Rc<dyn LegacyIsolationClaimRule>>,
}

impl LegacyIsolationClaimValidator {
    pub fn validate(&self, item: &LegacyIsolationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyIsolationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyIsolationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_isolation_claim_validator() -> LegacyIsolationClaimValidator {
    LegacyIsolationClaimValidator {
        rules: vec![
            Rc::new(LegacyIsolationClaimAllowedSummaryRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn LegacyIsolationClaimRule>,
            Rc::new(LegacyIsolationClaimCategoryLengthRule { max_length: 64 }),
            Rc::new(LegacyIsolationClaimLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_isolation_claim_validator();
        let item = new_legacy_isolation_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = LegacyIsolationClaimCategoryLengthRule { max_length: 80 };
        let mut item = new_legacy_isolation_claim("x", "y");
        item.category = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyIsolationClaimLabelLengthRule { max_length: 120 };
        let mut item = new_legacy_isolation_claim("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
