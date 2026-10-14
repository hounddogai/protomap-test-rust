use super::*;

use acme_kit::text;

/// Where a audit event preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AuditEventPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for AuditEventPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPreference {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub rate: f64,
    pub owner_id: String,
    pub archived: bool,
    pub duration_minutes: i64,
    pub quantity: i64,
    pub unit: String,
    pub window_days: i64,
    pub status: AuditEventPreferenceStatus,
}

impl AuditEventPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventPreferenceStatus) -> bool {
        match self.status {
            AuditEventPreferenceStatus::Idle => {
                match next {
                    AuditEventPreferenceStatus::Warming | AuditEventPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventPreferenceStatus::Warming => {
                match next {
                    AuditEventPreferenceStatus::Draining | AuditEventPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventPreferenceStatus::Ready => {
                match next {
                    AuditEventPreferenceStatus::Draining | AuditEventPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventPreferenceStatus::Draining => {
                match next {
                    AuditEventPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AuditEventPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventPreference {
        AuditEventPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            rate: self.rate,
            owner_id: self.owner_id.clone(),
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            quantity: self.quantity,
            unit: self.unit.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a audit event preference in status idle.
pub fn new_audit_event_preference(code: &str, name: &str) -> AuditEventPreference {
    AuditEventPreference {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        rate: 0.0,
        owner_id: "".to_string(),
        archived: false,
        duration_minutes: 0,
        quantity: 0,
        unit: "".to_string(),
        window_days: 0,
        status: AuditEventPreferenceStatus::Idle,
    }
}
