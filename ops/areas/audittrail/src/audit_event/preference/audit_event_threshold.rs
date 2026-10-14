use super::*;

use acme_kit::text;

/// Where a audit event threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AuditEventThresholdStatus {
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

impl std::fmt::Display for AuditEventThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event threshold as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventThreshold {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub source: String,
    pub attempts: i64,
    pub port: i64,
    pub status: AuditEventThresholdStatus,
}

impl AuditEventThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventThresholdStatus) -> bool {
        match self.status {
            AuditEventThresholdStatus::Idle => {
                return next == AuditEventThresholdStatus::Ready || next == AuditEventThresholdStatus::Stopped;
            }
            AuditEventThresholdStatus::Warming => {
                match next {
                    AuditEventThresholdStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventThresholdStatus::Ready => {
                match next {
                    AuditEventThresholdStatus::Draining | AuditEventThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventThresholdStatus::Draining => {
                return next == AuditEventThresholdStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.attempts as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventThreshold {
        AuditEventThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            source: self.source.clone(),
            attempts: self.attempts,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a audit event threshold in status idle.
pub fn new_audit_event_threshold(code: &str, name: &str) -> AuditEventThreshold {
    AuditEventThreshold {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        source: "".to_string(),
        attempts: 0,
        port: 0,
        status: AuditEventThresholdStatus::Idle,
    }
}
