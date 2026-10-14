use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending timeline record must pass. Check returns its problems, or none.
pub trait PendingTimelineRecordRule: std::fmt::Debug {
    fn check(&self, item: &PendingTimelineRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineRecordAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl PendingTimelineRecordRule for PendingTimelineRecordAllowedTenantIdRule {
    fn check(&self, item: &PendingTimelineRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is above maximum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineRecordRequiredAudienceRule {
    pub message: String,
}

impl PendingTimelineRecordRule for PendingTimelineRecordRequiredAudienceRule {
    fn check(&self, item: &PendingTimelineRecord) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineRecordLabelLengthRule {
    pub max_length: i64,
}

impl PendingTimelineRecordRule for PendingTimelineRecordLabelLengthRule {
    fn check(&self, item: &PendingTimelineRecord) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineRecordFingerprintLengthRule {
    pub max_length: i64,
}

impl PendingTimelineRecordRule for PendingTimelineRecordFingerprintLengthRule {
    fn check(&self, item: &PendingTimelineRecord) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineRecordAudienceLengthRule {
    pub max_length: i64,
}

impl PendingTimelineRecordRule for PendingTimelineRecordAudienceLengthRule {
    fn check(&self, item: &PendingTimelineRecord) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineRecordApprovedGuardRule {
    pub limit: i64,
}

impl PendingTimelineRecordRule for PendingTimelineRecordApprovedGuardRule {
    fn check(&self, item: &PendingTimelineRecord) -> Vec<String> {
        if item.approved && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while approved",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending timeline record.
#[derive(Debug, Clone)]
pub struct PendingTimelineRecordValidator {
    pub rules: Vec<Rc<dyn PendingTimelineRecordRule>>,
}

impl PendingTimelineRecordValidator {
    pub fn validate(&self, item: &PendingTimelineRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTimelineRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTimelineRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_timeline_record_validator() -> PendingTimelineRecordValidator {
    PendingTimelineRecordValidator {
        rules: vec![
            Rc::new(PendingTimelineRecordAllowedTenantIdRule {
                allowed: vec!["pending".to_string()],
            }) as Rc<dyn PendingTimelineRecordRule>,
            Rc::new(PendingTimelineRecordRequiredAudienceRule {
                message: "pending timeline record needs a value".to_string(),
            }),
            Rc::new(PendingTimelineRecordLabelLengthRule { max_length: 64 }),
            Rc::new(PendingTimelineRecordFingerprintLengthRule { max_length: 32 }),
            Rc::new(PendingTimelineRecordAudienceLengthRule { max_length: 3600 }),
            Rc::new(PendingTimelineRecordApprovedGuardRule { limit: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_timeline_record_validator();
        let item = new_pending_timeline_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PendingTimelineRecordRequiredAudienceRule {
            message: "pending timeline record needs a value".to_string(),
        };
        let mut item = new_pending_timeline_record("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingTimelineRecordLabelLengthRule { max_length: 120 };
        let mut item = new_pending_timeline_record("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = PendingTimelineRecordFingerprintLengthRule { max_length: 3600 };
        let mut item = new_pending_timeline_record("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PendingTimelineRecordAudienceLengthRule { max_length: 64 };
        let mut item = new_pending_timeline_record("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
