use super::*;

use acme_kit::text;

/// Where a audit event baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AuditEventBaselineStatus {
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

impl std::fmt::Display for AuditEventBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBaseline {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub approved: bool,
    pub quantity: i64,
    pub currency: String,
    pub rotation_days: i64,
    pub created_at: i64,
    pub minimum: f64,
    pub rate: f64,
    pub summary: String,
    pub status: AuditEventBaselineStatus,
}

impl AuditEventBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventBaselineStatus) -> bool {
        match self.status {
            AuditEventBaselineStatus::Idle => {
                match next {
                    AuditEventBaselineStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventBaselineStatus::Warming => {
                return next == AuditEventBaselineStatus::Ready || next == AuditEventBaselineStatus::Draining;
            }
            AuditEventBaselineStatus::Ready => {
                return next == AuditEventBaselineStatus::Draining || next == AuditEventBaselineStatus::Stopped;
            }
            AuditEventBaselineStatus::Draining => {
                return next == AuditEventBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event baseline {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.rotation_days as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventBaseline {
        AuditEventBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            approved: self.approved,
            quantity: self.quantity,
            currency: self.currency.clone(),
            rotation_days: self.rotation_days,
            created_at: self.created_at,
            minimum: self.minimum,
            rate: self.rate,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event baseline in status idle.
pub fn new_audit_event_baseline(code: &str, title: &str) -> AuditEventBaseline {
    AuditEventBaseline {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        approved: false,
        quantity: 0,
        currency: "".to_string(),
        rotation_days: 0,
        created_at: 0,
        minimum: 0.0,
        rate: 0.0,
        summary: "".to_string(),
        status: AuditEventBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_audit_event_baseline("code-231", "title-257");
        assert_eq!(item.status, AuditEventBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_audit_event_baseline("code-231", "title-257");
        if item.move_to(AuditEventBaselineStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventBaselineStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_audit_event_baseline("code-231", "title-257");
        item.status = AuditEventBaselineStatus::Stopped;
        assert!(
            item.move_to(AuditEventBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
