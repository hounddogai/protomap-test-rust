use super::*;

use acme_kit::text;

/// Where a fallback approval digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackApprovalDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackApprovalDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackApprovalDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback approval digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalDigest {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub external_ref: String,
    pub category: String,
    pub source: String,
    pub minimum: f64,
    pub contact_email: String,
    pub status: FallbackApprovalDigestStatus,
}

impl FallbackApprovalDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackApprovalDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackApprovalDigestStatus) -> bool {
        match self.status {
            FallbackApprovalDigestStatus::Requested => {
                match next {
                    FallbackApprovalDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackApprovalDigestStatus::Granted => {
                return next == FallbackApprovalDigestStatus::Revoked || next == FallbackApprovalDigestStatus::Expired;
            }
            FallbackApprovalDigestStatus::Revoked => {
                return next == FallbackApprovalDigestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackApprovalDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback approval digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback approval digest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> FallbackApprovalDigest {
        FallbackApprovalDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            external_ref: self.external_ref.clone(),
            category: self.category.clone(),
            source: self.source.clone(),
            minimum: self.minimum,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback approval digest in status requested.
pub fn new_fallback_approval_digest(code: &str, label: &str) -> FallbackApprovalDigest {
    FallbackApprovalDigest {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        external_ref: "".to_string(),
        category: "".to_string(),
        source: "".to_string(),
        minimum: 0.0,
        contact_email: "".to_string(),
        status: FallbackApprovalDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_approval_digest("code-810", "label-40");
        assert_eq!(item.status, FallbackApprovalDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_fallback_approval_digest("code-810", "label-40");
        if item.move_to(FallbackApprovalDigestStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackApprovalDigestStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_approval_digest("code-810", "label-40");
        item.status = FallbackApprovalDigestStatus::Expired;
        assert!(
            item.move_to(FallbackApprovalDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
