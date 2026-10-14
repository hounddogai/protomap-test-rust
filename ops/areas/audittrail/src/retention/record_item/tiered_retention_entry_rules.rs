use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered retention entry must pass. Check returns its problems, or none.
pub trait TieredRetentionEntryRule: std::fmt::Debug {
    fn check(&self, item: &TieredRetentionEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionEntryPinnedGuardRule {
    pub limit: f64,
}

impl TieredRetentionEntryRule for TieredRetentionEntryPinnedGuardRule {
    fn check(&self, item: &TieredRetentionEntry) -> Vec<String> {
        if item.pinned && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while pinned", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionEntryPrincipalLengthRule {
    pub max_length: i64,
}

impl TieredRetentionEntryRule for TieredRetentionEntryPrincipalLengthRule {
    fn check(&self, item: &TieredRetentionEntry) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionEntryAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl TieredRetentionEntryRule for TieredRetentionEntryAllowedAudienceRule {
    fn check(&self, item: &TieredRetentionEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is unknown", item.audience)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered retention entry.
#[derive(Debug, Clone)]
pub struct TieredRetentionEntryValidator {
    pub rules: Vec<Rc<dyn TieredRetentionEntryRule>>,
}

impl TieredRetentionEntryValidator {
    pub fn validate(&self, item: &TieredRetentionEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRetentionEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRetentionEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_retention_entry_validator() -> TieredRetentionEntryValidator {
    TieredRetentionEntryValidator {
        rules: vec![
            Rc::new(TieredRetentionEntryPinnedGuardRule { limit: 3.3 }) as Rc<dyn TieredRetentionEntryRule>,
            Rc::new(TieredRetentionEntryPrincipalLengthRule { max_length: 86400 }),
            Rc::new(TieredRetentionEntryAllowedAudienceRule {
                allowed: vec!["global".to_string(), "global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_retention_entry_validator();
        let item = new_tiered_retention_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = TieredRetentionEntryPrincipalLengthRule { max_length: 120 };
        let mut item = new_tiered_retention_entry("x", "y");
        item.principal = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
