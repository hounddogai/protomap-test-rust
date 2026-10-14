use super::*;

use acme_kit::text;

/// Where a credential notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CredentialNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CredentialNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialNotice {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub threshold: f64,
    pub retries: i64,
    pub segment: String,
    pub duration_minutes: i64,
    pub channel: String,
    pub contact_email: String,
    pub status: CredentialNoticeStatus,
}

impl CredentialNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialNoticeStatus) -> bool {
        match self.status {
            CredentialNoticeStatus::Proposed => {
                match next {
                    CredentialNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialNoticeStatus::Approved => {
                return next == CredentialNoticeStatus::Rejected || next == CredentialNoticeStatus::Withdrawn;
            }
            CredentialNoticeStatus::Rejected => {
                return next == CredentialNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.retries as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialNotice {
        CredentialNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            threshold: self.threshold,
            retries: self.retries,
            segment: self.segment.clone(),
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a credential notice in status proposed.
pub fn new_credential_notice(code: &str, name: &str) -> CredentialNotice {
    CredentialNotice {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        threshold: 0.0,
        retries: 0,
        segment: "".to_string(),
        duration_minutes: 0,
        channel: "".to_string(),
        contact_email: "".to_string(),
        status: CredentialNoticeStatus::Proposed,
    }
}
