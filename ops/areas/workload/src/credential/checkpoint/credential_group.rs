use super::*;

use acme_kit::text;

/// Where a credential group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CredentialGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CredentialGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialGroup {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub unit: String,
    pub budget: f64,
    pub expires_at: i64,
    pub priority: i64,
    pub host: String,
    pub status: CredentialGroupStatus,
}

impl CredentialGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialGroupStatus) -> bool {
        match self.status {
            CredentialGroupStatus::Proposed => {
                return next == CredentialGroupStatus::Rejected || next == CredentialGroupStatus::Withdrawn;
            }
            CredentialGroupStatus::Approved => {
                return next == CredentialGroupStatus::Withdrawn;
            }
            CredentialGroupStatus::Rejected => {
                return next == CredentialGroupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.priority as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialGroup {
        CredentialGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            unit: self.unit.clone(),
            budget: self.budget,
            expires_at: self.expires_at,
            priority: self.priority,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a credential group in status proposed.
pub fn new_credential_group(code: &str, label: &str) -> CredentialGroup {
    CredentialGroup {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        unit: "".to_string(),
        budget: 0.0,
        expires_at: 0,
        priority: 0,
        host: "".to_string(),
        status: CredentialGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_credential_group("code-783", "label-725");
        assert_eq!(item.status, CredentialGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_credential_group("code-783", "label-725");
        if item.move_to(CredentialGroupStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialGroupStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_credential_group("code-783", "label-725");
        item.status = CredentialGroupStatus::Withdrawn;
        assert!(
            item.move_to(CredentialGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
