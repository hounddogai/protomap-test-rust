use super::*;

use acme_kit::text;

/// Where a identity threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl IdentityThresholdStatus {
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

impl std::fmt::Display for IdentityThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityThreshold {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub host: String,
    pub namespace: String,
    pub weight: f64,
    pub unit: String,
    pub source: String,
    pub tenant_id: String,
    pub pinned: bool,
    pub created_at: i64,
    pub status: IdentityThresholdStatus,
}

impl IdentityThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityThresholdStatus) -> bool {
        match self.status {
            IdentityThresholdStatus::Open => {
                match next {
                    IdentityThresholdStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityThresholdStatus::InReview => {
                return next == IdentityThresholdStatus::Resolved || next == IdentityThresholdStatus::Reopened;
            }
            IdentityThresholdStatus::Resolved => {
                return next == IdentityThresholdStatus::Reopened || next == IdentityThresholdStatus::Archived;
            }
            IdentityThresholdStatus::Reopened => {
                return next == IdentityThresholdStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity threshold {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.created_at as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityThreshold {
        IdentityThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            host: self.host.clone(),
            namespace: self.namespace.clone(),
            weight: self.weight,
            unit: self.unit.clone(),
            source: self.source.clone(),
            tenant_id: self.tenant_id.clone(),
            pinned: self.pinned,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a identity threshold in status open.
pub fn new_identity_threshold(code: &str, label: &str) -> IdentityThreshold {
    IdentityThreshold {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        host: "".to_string(),
        namespace: "".to_string(),
        weight: 0.0,
        unit: "".to_string(),
        source: "".to_string(),
        tenant_id: "".to_string(),
        pinned: false,
        created_at: 0,
        status: IdentityThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_identity_threshold("code-826", "label-874");
        assert_eq!(item.status, IdentityThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_identity_threshold("code-826", "label-874");
        if item.move_to(IdentityThresholdStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityThresholdStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_identity_threshold("code-826", "label-874");
        item.status = IdentityThresholdStatus::Archived;
        assert!(
            item.move_to(IdentityThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
