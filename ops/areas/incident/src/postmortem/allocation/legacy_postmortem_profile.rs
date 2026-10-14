use super::*;

use acme_kit::text;

/// Where a legacy postmortem profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPostmortemProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyPostmortemProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyPostmortemProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy postmortem profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemProfile {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub score: f64,
    pub rate: f64,
    pub scope: String,
    pub segment: String,
    pub contact_email: String,
    pub limit: i64,
    pub floor_value: f64,
    pub status: LegacyPostmortemProfileStatus,
}

impl LegacyPostmortemProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPostmortemProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPostmortemProfileStatus) -> bool {
        match self.status {
            LegacyPostmortemProfileStatus::Requested => {
                return next == LegacyPostmortemProfileStatus::Granted;
            }
            LegacyPostmortemProfileStatus::Granted => {
                match next {
                    LegacyPostmortemProfileStatus::Revoked | LegacyPostmortemProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPostmortemProfileStatus::Revoked => {
                match next {
                    LegacyPostmortemProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyPostmortemProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy postmortem profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.priority as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPostmortemProfile {
        LegacyPostmortemProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            score: self.score,
            rate: self.rate,
            scope: self.scope.clone(),
            segment: self.segment.clone(),
            contact_email: self.contact_email.clone(),
            limit: self.limit,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy postmortem profile in status requested.
pub fn new_legacy_postmortem_profile(code: &str, label: &str) -> LegacyPostmortemProfile {
    LegacyPostmortemProfile {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        score: 0.0,
        rate: 0.0,
        scope: "".to_string(),
        segment: "".to_string(),
        contact_email: "".to_string(),
        limit: 0,
        floor_value: 0.0,
        status: LegacyPostmortemProfileStatus::Requested,
    }
}
