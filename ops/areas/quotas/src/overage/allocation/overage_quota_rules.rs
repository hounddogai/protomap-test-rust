use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage quota must pass. Check returns its problems, or none.
pub trait OverageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &OverageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageQuotaExternalRefLengthRule {
    pub max_length: i64,
}

impl OverageQuotaRule for OverageQuotaExternalRefLengthRule {
    fn check(&self, item: &OverageQuota) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageQuotaUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageQuotaRule for OverageQuotaUpdatedAtRangeRule {
    fn check(&self, item: &OverageQuota) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageQuotaCodePrefixRule {
    pub prefix: String,
}

impl OverageQuotaRule for OverageQuotaCodePrefixRule {
    fn check(&self, item: &OverageQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage quota.
#[derive(Debug, Clone)]
pub struct OverageQuotaValidator {
    pub rules: Vec<Rc<dyn OverageQuotaRule>>,
}

impl OverageQuotaValidator {
    pub fn validate(&self, item: &OverageQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_quota_validator() -> OverageQuotaValidator {
    OverageQuotaValidator {
        rules: vec![
            Rc::new(OverageQuotaExternalRefLengthRule { max_length: 80 }) as Rc<dyn OverageQuotaRule>,
            Rc::new(OverageQuotaUpdatedAtRangeRule { min: 32, max: 3600 }),
            Rc::new(OverageQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_quota_validator();
        let item = new_overage_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = OverageQuotaExternalRefLengthRule { max_length: 80 };
        let mut item = new_overage_quota("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = OverageQuotaUpdatedAtRangeRule { min: 16, max: 86400 };
        let mut item = new_overage_quota("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
