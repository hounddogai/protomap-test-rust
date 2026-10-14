use super::*;

use acme_kit::text;

/// Where a credential estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for CredentialEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEstimate {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub host: String,
    pub key_id: String,
    pub margin: f64,
    pub amount: f64,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub ratio: f64,
    pub ceiling: f64,
    pub status: CredentialEstimateStatus,
}

impl CredentialEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialEstimateStatus) -> bool {
        match self.status {
            CredentialEstimateStatus::Open => {
                return next == CredentialEstimateStatus::InReview || next == CredentialEstimateStatus::Archived;
            }
            CredentialEstimateStatus::InReview => {
                match next {
                    CredentialEstimateStatus::Resolved | CredentialEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialEstimateStatus::Resolved => {
                match next {
                    CredentialEstimateStatus::Reopened | CredentialEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialEstimateStatus::Reopened => {
                return next == CredentialEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential estimate", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ratio + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialEstimate {
        CredentialEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            host: self.host.clone(),
            key_id: self.key_id.clone(),
            margin: self.margin,
            amount: self.amount,
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            ratio: self.ratio,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a credential estimate in status open.
pub fn new_credential_estimate(code: &str, label: &str) -> CredentialEstimate {
    CredentialEstimate {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        host: "".to_string(),
        key_id: "".to_string(),
        margin: 0.0,
        amount: 0.0,
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        ratio: 0.0,
        ceiling: 0.0,
        status: CredentialEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_credential_estimate("code-347", "label-22");
        assert_eq!(item.status, CredentialEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_credential_estimate("code-347", "label-22");
        if item.move_to(CredentialEstimateStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialEstimateStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_credential_estimate("code-347", "label-22");
        item.status = CredentialEstimateStatus::Archived;
        assert!(
            item.move_to(CredentialEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
