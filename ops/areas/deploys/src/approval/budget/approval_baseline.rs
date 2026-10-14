use super::*;

use acme_kit::text;

/// Where a approval baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ApprovalBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ApprovalBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval baseline as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalBaseline {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub country: String,
    pub key_id: String,
    pub version: i64,
    pub notes: String,
    pub minimum: f64,
    pub channel: String,
    pub sequence: i64,
    pub category: String,
    pub status: ApprovalBaselineStatus,
}

impl ApprovalBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalBaselineStatus) -> bool {
        match self.status {
            ApprovalBaselineStatus::Requested => {
                match next {
                    ApprovalBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalBaselineStatus::Granted => {
                return next == ApprovalBaselineStatus::Expired;
            }
            ApprovalBaselineStatus::Revoked => {
                match next {
                    ApprovalBaselineStatus::Expired => {
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

    pub fn move_to(&mut self, next: ApprovalBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval baseline", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.minimum + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalBaseline {
        ApprovalBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            country: self.country.clone(),
            key_id: self.key_id.clone(),
            version: self.version,
            notes: self.notes.clone(),
            minimum: self.minimum,
            channel: self.channel.clone(),
            sequence: self.sequence,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a approval baseline in status requested.
pub fn new_approval_baseline(code: &str, title: &str) -> ApprovalBaseline {
    ApprovalBaseline {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        country: "".to_string(),
        key_id: "".to_string(),
        version: 0,
        notes: "".to_string(),
        minimum: 0.0,
        channel: "".to_string(),
        sequence: 0,
        category: "".to_string(),
        status: ApprovalBaselineStatus::Requested,
    }
}
