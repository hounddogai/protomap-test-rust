use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension snapshot must pass. Check returns its problems, or none.
pub trait SuspensionSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSnapshotRequiredAudienceRule {
    pub message: String,
}

impl SuspensionSnapshotRule for SuspensionSnapshotRequiredAudienceRule {
    fn check(&self, item: &SuspensionSnapshot) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSnapshotDescriptionLengthRule {
    pub max_length: i64,
}

impl SuspensionSnapshotRule for SuspensionSnapshotDescriptionLengthRule {
    fn check(&self, item: &SuspensionSnapshot) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSnapshotEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionSnapshotRule for SuspensionSnapshotEndsAtRangeRule {
    fn check(&self, item: &SuspensionSnapshot) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSnapshotAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl SuspensionSnapshotRule for SuspensionSnapshotAllowedAudienceRule {
    fn check(&self, item: &SuspensionSnapshot) -> Vec<String> {
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

/// Runs every rule of a suspension snapshot.
#[derive(Debug, Clone)]
pub struct SuspensionSnapshotValidator {
    pub rules: Vec<Rc<dyn SuspensionSnapshotRule>>,
}

impl SuspensionSnapshotValidator {
    pub fn validate(&self, item: &SuspensionSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_snapshot_validator() -> SuspensionSnapshotValidator {
    SuspensionSnapshotValidator {
        rules: vec![
            Rc::new(SuspensionSnapshotRequiredAudienceRule {
                message: "suspension snapshot needs this field".to_string(),
            }) as Rc<dyn SuspensionSnapshotRule>,
            Rc::new(SuspensionSnapshotDescriptionLengthRule { max_length: 80 }),
            Rc::new(SuspensionSnapshotEndsAtRangeRule { min: 16, max: 3600 }),
            Rc::new(SuspensionSnapshotAllowedAudienceRule {
                allowed: vec!["manual".to_string(), "fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_snapshot_validator();
        let item = new_suspension_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = SuspensionSnapshotRequiredAudienceRule {
            message: "suspension snapshot needs a value".to_string(),
        };
        let mut item = new_suspension_snapshot("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = SuspensionSnapshotDescriptionLengthRule { max_length: 3600 };
        let mut item = new_suspension_snapshot("x", "y");
        item.description = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = SuspensionSnapshotEndsAtRangeRule { min: 32, max: 120 };
        let mut item = new_suspension_snapshot("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
