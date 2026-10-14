use super::*;

use acme_kit::text;

/// Where a escalation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EscalationAdjustmentStatus {
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

impl std::fmt::Display for EscalationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAdjustment {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub summary: String,
    pub notes: String,
    pub contact_email: String,
    pub expires_at: i64,
    pub host: String,
    pub quota_units: i64,
    pub retries: i64,
    pub segment: String,
    pub status: EscalationAdjustmentStatus,
}

impl EscalationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationAdjustmentStatus) -> bool {
        match self.status {
            EscalationAdjustmentStatus::Idle => {
                match next {
                    EscalationAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAdjustmentStatus::Warming => {
                match next {
                    EscalationAdjustmentStatus::Ready | EscalationAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAdjustmentStatus::Ready => {
                return next == EscalationAdjustmentStatus::Stopped;
            }
            EscalationAdjustmentStatus::Draining => {
                match next {
                    EscalationAdjustmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EscalationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation adjustment {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.sequence as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationAdjustment {
        EscalationAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            summary: self.summary.clone(),
            notes: self.notes.clone(),
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            host: self.host.clone(),
            quota_units: self.quota_units,
            retries: self.retries,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a escalation adjustment in status idle.
pub fn new_escalation_adjustment(code: &str, label: &str) -> EscalationAdjustment {
    EscalationAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        summary: "".to_string(),
        notes: "".to_string(),
        contact_email: "".to_string(),
        expires_at: 0,
        host: "".to_string(),
        quota_units: 0,
        retries: 0,
        segment: "".to_string(),
        status: EscalationAdjustmentStatus::Idle,
    }
}
