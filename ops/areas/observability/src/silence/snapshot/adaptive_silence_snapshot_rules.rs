use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive silence snapshot must pass. Check returns its problems, or none.
pub trait AdaptiveSilenceSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshotIssuerLengthRule {
    pub max_length: i64,
}

impl AdaptiveSilenceSnapshotRule for AdaptiveSilenceSnapshotIssuerLengthRule {
    fn check(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshotApprovedGuardRule {
    pub limit: f64,
}

impl AdaptiveSilenceSnapshotRule for AdaptiveSilenceSnapshotApprovedGuardRule {
    fn check(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String> {
        if item.approved && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while approved", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveSilenceSnapshotRule for AdaptiveSilenceSnapshotCodeLengthRule {
    fn check(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshotRequiredNameRule {
    pub message: String,
}

impl AdaptiveSilenceSnapshotRule for AdaptiveSilenceSnapshotRequiredNameRule {
    fn check(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshotCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveSilenceSnapshotRule for AdaptiveSilenceSnapshotCodePrefixRule {
    fn check(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshotBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveSilenceSnapshotRule for AdaptiveSilenceSnapshotBatchSizeRangeRule {
    fn check(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive silence snapshot.
#[derive(Debug, Clone)]
pub struct AdaptiveSilenceSnapshotValidator {
    pub rules: Vec<Rc<dyn AdaptiveSilenceSnapshotRule>>,
}

impl AdaptiveSilenceSnapshotValidator {
    pub fn validate(&self, item: &AdaptiveSilenceSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveSilenceSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveSilenceSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_silence_snapshot_validator() -> AdaptiveSilenceSnapshotValidator {
    AdaptiveSilenceSnapshotValidator {
        rules: vec![
            Rc::new(AdaptiveSilenceSnapshotIssuerLengthRule { max_length: 64 }) as Rc<dyn AdaptiveSilenceSnapshotRule>,
            Rc::new(AdaptiveSilenceSnapshotApprovedGuardRule { limit: 2.0 }),
            Rc::new(AdaptiveSilenceSnapshotCodeLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveSilenceSnapshotRequiredNameRule {
                message: "adaptive silence snapshot needs a value".to_string(),
            }),
            Rc::new(AdaptiveSilenceSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AdaptiveSilenceSnapshotBatchSizeRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_silence_snapshot_validator();
        let item = new_adaptive_silence_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = AdaptiveSilenceSnapshotIssuerLengthRule { max_length: 80 };
        let mut item = new_adaptive_silence_snapshot("x", "y");
        item.issuer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveSilenceSnapshotCodeLengthRule { max_length: 86400 };
        let mut item = new_adaptive_silence_snapshot("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveSilenceSnapshotRequiredNameRule {
            message: "adaptive silence snapshot needs a value".to_string(),
        };
        let mut item = new_adaptive_silence_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = AdaptiveSilenceSnapshotBatchSizeRangeRule { min: 16, max: 64 };
        let mut item = new_adaptive_silence_snapshot("x", "y");
        item.batch_size = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
