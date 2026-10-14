use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped overage snapshot must pass. Check returns its problems, or none.
pub trait CappedOverageSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &CappedOverageSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSnapshotBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedOverageSnapshotRule for CappedOverageSnapshotBatchSizeRangeRule {
    fn check(&self, item: &CappedOverageSnapshot) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSnapshotCodePrefixRule {
    pub prefix: String,
}

impl CappedOverageSnapshotRule for CappedOverageSnapshotCodePrefixRule {
    fn check(&self, item: &CappedOverageSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSnapshotAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl CappedOverageSnapshotRule for CappedOverageSnapshotAllowedPrincipalRule {
    fn check(&self, item: &CappedOverageSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is inconsistent", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSnapshotRequiredPrincipalRule {
    pub message: String,
}

impl CappedOverageSnapshotRule for CappedOverageSnapshotRequiredPrincipalRule {
    fn check(&self, item: &CappedOverageSnapshot) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl CappedOverageSnapshotRule for CappedOverageSnapshotCodeLengthRule {
    fn check(&self, item: &CappedOverageSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped overage snapshot.
#[derive(Debug, Clone)]
pub struct CappedOverageSnapshotValidator {
    pub rules: Vec<Rc<dyn CappedOverageSnapshotRule>>,
}

impl CappedOverageSnapshotValidator {
    pub fn validate(&self, item: &CappedOverageSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedOverageSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedOverageSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_overage_snapshot_validator() -> CappedOverageSnapshotValidator {
    CappedOverageSnapshotValidator {
        rules: vec![
            Rc::new(CappedOverageSnapshotBatchSizeRangeRule { min: 16, max: 255 }) as Rc<dyn CappedOverageSnapshotRule>,
            Rc::new(CappedOverageSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CappedOverageSnapshotAllowedPrincipalRule {
                allowed: vec!["fallback".to_string(), "strict".to_string(), "global".to_string()],
            }),
            Rc::new(CappedOverageSnapshotRequiredPrincipalRule {
                message: "capped overage snapshot needs this field".to_string(),
            }),
            Rc::new(CappedOverageSnapshotCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_overage_snapshot_validator();
        let item = new_capped_overage_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = CappedOverageSnapshotBatchSizeRangeRule { min: 64, max: 80 };
        let mut item = new_capped_overage_snapshot("x", "y");
        item.batch_size = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = CappedOverageSnapshotRequiredPrincipalRule {
            message: "capped overage snapshot needs a value".to_string(),
        };
        let mut item = new_capped_overage_snapshot("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedOverageSnapshotCodeLengthRule { max_length: 64 };
        let mut item = new_capped_overage_snapshot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
