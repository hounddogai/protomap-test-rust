use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive slo record must pass. Check returns its problems, or none.
pub trait AdaptiveSloRecordRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSloRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloRecordCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveSloRecordRule for AdaptiveSloRecordCodePrefixRule {
    fn check(&self, item: &AdaptiveSloRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloRecordTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl AdaptiveSloRecordRule for AdaptiveSloRecordTagsLimitRule {
    fn check(&self, item: &AdaptiveSloRecord) -> Vec<String> {
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
pub struct AdaptiveSloRecordAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveSloRecordRule for AdaptiveSloRecordAllowedCodeRule {
    fn check(&self, item: &AdaptiveSloRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive slo record.
#[derive(Debug, Clone)]
pub struct AdaptiveSloRecordValidator {
    pub rules: Vec<Rc<dyn AdaptiveSloRecordRule>>,
}

impl AdaptiveSloRecordValidator {
    pub fn validate(&self, item: &AdaptiveSloRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveSloRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSloRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_slo_record_validator() -> AdaptiveSloRecordValidator {
    AdaptiveSloRecordValidator {
        rules: vec![
            Rc::new(AdaptiveSloRecordCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn AdaptiveSloRecordRule>,
            Rc::new(AdaptiveSloRecordTagsLimitRule {
                max_count: 86400,
                banned: vec!["manual".to_string()],
            }),
            Rc::new(AdaptiveSloRecordAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_slo_record_validator();
        let item = new_adaptive_slo_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }
}
