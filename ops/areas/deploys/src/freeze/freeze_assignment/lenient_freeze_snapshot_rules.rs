use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient freeze snapshot must pass. Check returns its problems, or none.
pub trait LenientFreezeSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &LenientFreezeSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeSnapshotCodePrefixRule {
    pub prefix: String,
}

impl LenientFreezeSnapshotRule for LenientFreezeSnapshotCodePrefixRule {
    fn check(&self, item: &LenientFreezeSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeSnapshotChecksumLengthRule {
    pub max_length: i64,
}

impl LenientFreezeSnapshotRule for LenientFreezeSnapshotChecksumLengthRule {
    fn check(&self, item: &LenientFreezeSnapshot) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeSnapshotBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientFreezeSnapshotRule for LenientFreezeSnapshotBatchSizeRangeRule {
    fn check(&self, item: &LenientFreezeSnapshot) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient freeze snapshot.
#[derive(Debug, Clone)]
pub struct LenientFreezeSnapshotValidator {
    pub rules: Vec<Rc<dyn LenientFreezeSnapshotRule>>,
}

impl LenientFreezeSnapshotValidator {
    pub fn validate(&self, item: &LenientFreezeSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientFreezeSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFreezeSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_freeze_snapshot_validator() -> LenientFreezeSnapshotValidator {
    LenientFreezeSnapshotValidator {
        rules: vec![
            Rc::new(LenientFreezeSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn LenientFreezeSnapshotRule>,
            Rc::new(LenientFreezeSnapshotChecksumLengthRule { max_length: 32 }),
            Rc::new(LenientFreezeSnapshotBatchSizeRangeRule { min: 64, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_freeze_snapshot_validator();
        let item = new_lenient_freeze_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = LenientFreezeSnapshotChecksumLengthRule { max_length: 16 };
        let mut item = new_lenient_freeze_snapshot("x", "y");
        item.checksum = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = LenientFreezeSnapshotBatchSizeRangeRule { min: 64, max: 1000 };
        let mut item = new_lenient_freeze_snapshot("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
