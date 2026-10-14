use super::*;

use acme_kit::text;

/// Where a legacy ttl policy schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTtlPolicyScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyTtlPolicyScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyTtlPolicyScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy ttl policy schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTtlPolicySchedule {
    pub code: String,
    pub name: String,
    pub country: String,
    pub quota_units: i64,
    pub region: String,
    pub notes: String,
    pub ceiling: f64,
    pub score: f64,
    pub source: String,
    pub principal: String,
    pub threshold: f64,
    pub status: LegacyTtlPolicyScheduleStatus,
}

impl LegacyTtlPolicySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTtlPolicyScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTtlPolicyScheduleStatus) -> bool {
        match self.status {
            LegacyTtlPolicyScheduleStatus::Proposed => {
                match next {
                    LegacyTtlPolicyScheduleStatus::Rejected | LegacyTtlPolicyScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTtlPolicyScheduleStatus::Approved => {
                return next == LegacyTtlPolicyScheduleStatus::Rejected;
            }
            LegacyTtlPolicyScheduleStatus::Rejected => {
                match next {
                    LegacyTtlPolicyScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyTtlPolicyScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy ttl policy schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        (self.score * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTtlPolicySchedule {
        LegacyTtlPolicySchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            quota_units: self.quota_units,
            region: self.region.clone(),
            notes: self.notes.clone(),
            ceiling: self.ceiling,
            score: self.score,
            source: self.source.clone(),
            principal: self.principal.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy ttl policy schedule in status proposed.
pub fn new_legacy_ttl_policy_schedule(code: &str, name: &str) -> LegacyTtlPolicySchedule {
    LegacyTtlPolicySchedule {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        quota_units: 0,
        region: "".to_string(),
        notes: "".to_string(),
        ceiling: 0.0,
        score: 0.0,
        source: "".to_string(),
        principal: "".to_string(),
        threshold: 0.0,
        status: LegacyTtlPolicyScheduleStatus::Proposed,
    }
}
