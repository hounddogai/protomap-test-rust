use super::*;

use acme_kit::text;

/// Where a credential adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CredentialAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CredentialAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAdjustment {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub label: String,
    pub channel: String,
    pub ends_at: i64,
    pub weight: f64,
    pub unit: String,
    pub retries: i64,
    pub fingerprint: String,
    pub status: CredentialAdjustmentStatus,
}

impl CredentialAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialAdjustmentStatus) -> bool {
        match self.status {
            CredentialAdjustmentStatus::Planned => {
                return next == CredentialAdjustmentStatus::Completed;
            }
            CredentialAdjustmentStatus::Scheduled => {
                return next == CredentialAdjustmentStatus::Completed || next == CredentialAdjustmentStatus::Cancelled;
            }
            CredentialAdjustmentStatus::InProgress => {
                return next == CredentialAdjustmentStatus::Cancelled;
            }
            CredentialAdjustmentStatus::Completed => {
                match next {
                    CredentialAdjustmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CredentialAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential adjustment", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.weight * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialAdjustment {
        CredentialAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            label: self.label.clone(),
            channel: self.channel.clone(),
            ends_at: self.ends_at,
            weight: self.weight,
            unit: self.unit.clone(),
            retries: self.retries,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a credential adjustment in status planned.
pub fn new_credential_adjustment(code: &str, name: &str) -> CredentialAdjustment {
    CredentialAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        label: "".to_string(),
        channel: "".to_string(),
        ends_at: 0,
        weight: 0.0,
        unit: "".to_string(),
        retries: 0,
        fingerprint: "".to_string(),
        status: CredentialAdjustmentStatus::Planned,
    }
}
