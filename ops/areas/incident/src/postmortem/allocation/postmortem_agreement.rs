use super::*;

use acme_kit::text;

/// Where a postmortem agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PostmortemAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PostmortemAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem agreement as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAgreement {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub description: String,
    pub approved: bool,
    pub key_id: String,
    pub unit: String,
    pub window_days: i64,
    pub tenant_id: String,
    pub created_at: i64,
    pub status: PostmortemAgreementStatus,
}

impl PostmortemAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemAgreementStatus) -> bool {
        match self.status {
            PostmortemAgreementStatus::Requested => {
                match next {
                    PostmortemAgreementStatus::Granted | PostmortemAgreementStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemAgreementStatus::Granted => {
                return next == PostmortemAgreementStatus::Revoked || next == PostmortemAgreementStatus::Expired;
            }
            PostmortemAgreementStatus::Revoked => {
                match next {
                    PostmortemAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: PostmortemAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.created_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemAgreement {
        PostmortemAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            description: self.description.clone(),
            approved: self.approved,
            key_id: self.key_id.clone(),
            unit: self.unit.clone(),
            window_days: self.window_days,
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a postmortem agreement in status requested.
pub fn new_postmortem_agreement(code: &str, title: &str) -> PostmortemAgreement {
    PostmortemAgreement {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        description: "".to_string(),
        approved: false,
        key_id: "".to_string(),
        unit: "".to_string(),
        window_days: 0,
        tenant_id: "".to_string(),
        created_at: 0,
        status: PostmortemAgreementStatus::Requested,
    }
}
