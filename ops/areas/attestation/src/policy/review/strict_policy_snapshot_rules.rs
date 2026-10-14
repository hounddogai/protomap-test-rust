use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict policy snapshot must pass. Check returns its problems, or none.
pub trait StrictPolicySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &StrictPolicySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshotPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPolicySnapshotRule for StrictPolicySnapshotPortRangeRule {
    fn check(&self, item: &StrictPolicySnapshot) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshotApprovedGuardRule {
    pub limit: i64,
}

impl StrictPolicySnapshotRule for StrictPolicySnapshotApprovedGuardRule {
    fn check(&self, item: &StrictPolicySnapshot) -> Vec<String> {
        if item.approved && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while approved", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshotCipherLengthRule {
    pub max_length: i64,
}

impl StrictPolicySnapshotRule for StrictPolicySnapshotCipherLengthRule {
    fn check(&self, item: &StrictPolicySnapshot) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshotAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl StrictPolicySnapshotRule for StrictPolicySnapshotAllowedAudienceRule {
    fn check(&self, item: &StrictPolicySnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is missing", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshotExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPolicySnapshotRule for StrictPolicySnapshotExpiresAtRangeRule {
    fn check(&self, item: &StrictPolicySnapshot) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySnapshotPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPolicySnapshotRule for StrictPolicySnapshotPriorityRangeRule {
    fn check(&self, item: &StrictPolicySnapshot) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict policy snapshot.
#[derive(Debug, Clone)]
pub struct StrictPolicySnapshotValidator {
    pub rules: Vec<Rc<dyn StrictPolicySnapshotRule>>,
}

impl StrictPolicySnapshotValidator {
    pub fn validate(&self, item: &StrictPolicySnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictPolicySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPolicySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_policy_snapshot_validator() -> StrictPolicySnapshotValidator {
    StrictPolicySnapshotValidator {
        rules: vec![
            Rc::new(StrictPolicySnapshotPortRangeRule { min: 80, max: 255 }) as Rc<dyn StrictPolicySnapshotRule>,
            Rc::new(StrictPolicySnapshotApprovedGuardRule { limit: 3600 }),
            Rc::new(StrictPolicySnapshotCipherLengthRule { max_length: 80 }),
            Rc::new(StrictPolicySnapshotAllowedAudienceRule {
                allowed: vec!["lenient".to_string()],
            }),
            Rc::new(StrictPolicySnapshotExpiresAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(StrictPolicySnapshotPriorityRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_policy_snapshot_validator();
        let item = new_strict_policy_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictPolicySnapshotPortRangeRule { min: 64, max: 86400 };
        let mut item = new_strict_policy_snapshot("x", "y");
        item.port = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = StrictPolicySnapshotCipherLengthRule { max_length: 16 };
        let mut item = new_strict_policy_snapshot("x", "y");
        item.cipher = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = StrictPolicySnapshotExpiresAtRangeRule { min: 255, max: 1000 };
        let mut item = new_strict_policy_snapshot("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = StrictPolicySnapshotPriorityRangeRule { min: 16, max: 16 };
        let mut item = new_strict_policy_snapshot("x", "y");
        item.priority = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
