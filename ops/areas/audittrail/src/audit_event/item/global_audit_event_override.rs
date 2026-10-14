use super::*;

use acme_kit::text;

/// Where a global audit event override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAuditEventOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalAuditEventOverrideStatus {
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

impl std::fmt::Display for GlobalAuditEventOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global audit event override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventOverride {
    pub code: String,
    pub label: String,
    pub name: String,
    pub window_days: i64,
    pub margin: f64,
    pub score: f64,
    pub version: i64,
    pub attempts: i64,
    pub ttl_seconds: i64,
    pub threshold: f64,
    pub status: GlobalAuditEventOverrideStatus,
}

impl GlobalAuditEventOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAuditEventOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAuditEventOverrideStatus) -> bool {
        match self.status {
            GlobalAuditEventOverrideStatus::Idle => {
                match next {
                    GlobalAuditEventOverrideStatus::Ready | GlobalAuditEventOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventOverrideStatus::Warming => {
                return next == GlobalAuditEventOverrideStatus::Stopped;
            }
            GlobalAuditEventOverrideStatus::Ready => {
                match next {
                    GlobalAuditEventOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventOverrideStatus::Draining => {
                match next {
                    GlobalAuditEventOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalAuditEventOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global audit event override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.score + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAuditEventOverride {
        GlobalAuditEventOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            margin: self.margin,
            score: self.score,
            version: self.version,
            attempts: self.attempts,
            ttl_seconds: self.ttl_seconds,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global audit event override in status idle.
pub fn new_global_audit_event_override(code: &str, label: &str) -> GlobalAuditEventOverride {
    GlobalAuditEventOverride {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        window_days: 0,
        margin: 0.0,
        score: 0.0,
        version: 0,
        attempts: 0,
        ttl_seconds: 0,
        threshold: 0.0,
        status: GlobalAuditEventOverrideStatus::Idle,
    }
}
