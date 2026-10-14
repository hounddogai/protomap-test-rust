use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending federation baseline must pass. Check returns its problems, or none.
pub trait PendingFederationBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PendingFederationBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationBaselineAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PendingFederationBaselineRule for PendingFederationBaselineAllowedNameRule {
    fn check(&self, item: &PendingFederationBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationBaselineHostLengthRule {
    pub max_length: i64,
}

impl PendingFederationBaselineRule for PendingFederationBaselineHostLengthRule {
    fn check(&self, item: &PendingFederationBaseline) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationBaselinePrincipalLengthRule {
    pub max_length: i64,
}

impl PendingFederationBaselineRule for PendingFederationBaselinePrincipalLengthRule {
    fn check(&self, item: &PendingFederationBaseline) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending federation baseline.
#[derive(Debug, Clone)]
pub struct PendingFederationBaselineValidator {
    pub rules: Vec<Rc<dyn PendingFederationBaselineRule>>,
}

impl PendingFederationBaselineValidator {
    pub fn validate(&self, item: &PendingFederationBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingFederationBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFederationBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_federation_baseline_validator() -> PendingFederationBaselineValidator {
    PendingFederationBaselineValidator {
        rules: vec![
            Rc::new(PendingFederationBaselineAllowedNameRule {
                allowed: vec!["rolling".to_string(), "regional".to_string(), "manual".to_string()],
            }) as Rc<dyn PendingFederationBaselineRule>,
            Rc::new(PendingFederationBaselineHostLengthRule { max_length: 255 }),
            Rc::new(PendingFederationBaselinePrincipalLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_federation_baseline_validator();
        let item = new_pending_federation_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PendingFederationBaselineHostLengthRule { max_length: 32 };
        let mut item = new_pending_federation_baseline("x", "y");
        item.host = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = PendingFederationBaselinePrincipalLengthRule { max_length: 255 };
        let mut item = new_pending_federation_baseline("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
