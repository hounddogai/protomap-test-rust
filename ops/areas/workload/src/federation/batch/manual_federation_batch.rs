use super::*;

use acme_kit::text;

/// Where a manual federation batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFederationBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualFederationBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualFederationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual federation batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFederationBatch {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub contact_email: String,
    pub scope: String,
    pub namespace: String,
    pub phone: String,
    pub port: i64,
    pub starts_at: i64,
    pub status: ManualFederationBatchStatus,
}

impl ManualFederationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFederationBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFederationBatchStatus) -> bool {
        match self.status {
            ManualFederationBatchStatus::Proposed => {
                return next == ManualFederationBatchStatus::Approved;
            }
            ManualFederationBatchStatus::Approved => {
                return next == ManualFederationBatchStatus::Withdrawn;
            }
            ManualFederationBatchStatus::Rejected => {
                match next {
                    ManualFederationBatchStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualFederationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual federation batch {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.starts_at as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFederationBatch {
        ManualFederationBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            contact_email: self.contact_email.clone(),
            scope: self.scope.clone(),
            namespace: self.namespace.clone(),
            phone: self.phone.clone(),
            port: self.port,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a manual federation batch in status proposed.
pub fn new_manual_federation_batch(code: &str, label: &str) -> ManualFederationBatch {
    ManualFederationBatch {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        contact_email: "".to_string(),
        scope: "".to_string(),
        namespace: "".to_string(),
        phone: "".to_string(),
        port: 0,
        starts_at: 0,
        status: ManualFederationBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_federation_batch("code-610", "label-119");
        assert_eq!(item.status, ManualFederationBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_manual_federation_batch("code-610", "label-119");
        if item.move_to(ManualFederationBatchStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFederationBatchStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_federation_batch("code-610", "label-119");
        item.status = ManualFederationBatchStatus::Withdrawn;
        assert!(
            item.move_to(ManualFederationBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
