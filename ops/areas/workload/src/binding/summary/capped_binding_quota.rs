use super::*;

use acme_kit::text;

/// Where a capped binding quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBindingQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedBindingQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedBindingQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped binding quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBindingQuota {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub fingerprint: String,
    pub summary: String,
    pub minimum: f64,
    pub margin: f64,
    pub visible: bool,
    pub checksum: String,
    pub archived: bool,
    pub status: CappedBindingQuotaStatus,
}

impl CappedBindingQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBindingQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBindingQuotaStatus) -> bool {
        match self.status {
            CappedBindingQuotaStatus::Requested => {
                match next {
                    CappedBindingQuotaStatus::Revoked | CappedBindingQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBindingQuotaStatus::Granted => {
                match next {
                    CappedBindingQuotaStatus::Revoked | CappedBindingQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBindingQuotaStatus::Revoked => {
                return next == CappedBindingQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedBindingQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped binding quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.minimum - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBindingQuota {
        CappedBindingQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            fingerprint: self.fingerprint.clone(),
            summary: self.summary.clone(),
            minimum: self.minimum,
            margin: self.margin,
            visible: self.visible,
            checksum: self.checksum.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped binding quota in status requested.
pub fn new_capped_binding_quota(code: &str, label: &str) -> CappedBindingQuota {
    CappedBindingQuota {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        fingerprint: "".to_string(),
        summary: "".to_string(),
        minimum: 0.0,
        margin: 0.0,
        visible: false,
        checksum: "".to_string(),
        archived: false,
        status: CappedBindingQuotaStatus::Requested,
    }
}
