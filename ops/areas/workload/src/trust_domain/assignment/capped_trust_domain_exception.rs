use super::*;

use acme_kit::text;

/// Where a capped trust domain exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTrustDomainExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedTrustDomainExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedTrustDomainExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped trust domain exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainException {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub starts_at: i64,
    pub namespace: String,
    pub minimum: f64,
    pub archived: bool,
    pub scope: String,
    pub discount: f64,
    pub status: CappedTrustDomainExceptionStatus,
}

impl CappedTrustDomainException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTrustDomainExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTrustDomainExceptionStatus) -> bool {
        match self.status {
            CappedTrustDomainExceptionStatus::Proposed => {
                match next {
                    CappedTrustDomainExceptionStatus::Approved | CappedTrustDomainExceptionStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTrustDomainExceptionStatus::Approved => {
                return next == CappedTrustDomainExceptionStatus::Rejected;
            }
            CappedTrustDomainExceptionStatus::Rejected => {
                return next == CappedTrustDomainExceptionStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTrustDomainExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped trust domain exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        (self.discount * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTrustDomainException {
        CappedTrustDomainException {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            namespace: self.namespace.clone(),
            minimum: self.minimum,
            archived: self.archived,
            scope: self.scope.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped trust domain exception in status proposed.
pub fn new_capped_trust_domain_exception(code: &str, label: &str) -> CappedTrustDomainException {
    CappedTrustDomainException {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        starts_at: 0,
        namespace: "".to_string(),
        minimum: 0.0,
        archived: false,
        scope: "".to_string(),
        discount: 0.0,
        status: CappedTrustDomainExceptionStatus::Proposed,
    }
}
