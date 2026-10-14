use super::*;

use acme_kit::text;

/// Where a rolling escalation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEscalationAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingEscalationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollingEscalationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling escalation audit as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationAudit {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub cipher: String,
    pub score: f64,
    pub status: RollingEscalationAuditStatus,
}

impl RollingEscalationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEscalationAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEscalationAuditStatus) -> bool {
        match self.status {
            RollingEscalationAuditStatus::Draft => {
                match next {
                    RollingEscalationAuditStatus::Pending | RollingEscalationAuditStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEscalationAuditStatus::Pending => {
                return next == RollingEscalationAuditStatus::Active;
            }
            RollingEscalationAuditStatus::Active => {
                return next == RollingEscalationAuditStatus::Suspended || next == RollingEscalationAuditStatus::Closed;
            }
            RollingEscalationAuditStatus::Suspended => {
                return next == RollingEscalationAuditStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingEscalationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling escalation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.score - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingEscalationAudit {
        RollingEscalationAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            cipher: self.cipher.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling escalation audit in status draft.
pub fn new_rolling_escalation_audit(code: &str, title: &str) -> RollingEscalationAudit {
    RollingEscalationAudit {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        cipher: "".to_string(),
        score: 0.0,
        status: RollingEscalationAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_escalation_audit("code-459", "title-986");
        assert_eq!(item.status, RollingEscalationAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rolling_escalation_audit("code-459", "title-986");
        if item.move_to(RollingEscalationAuditStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingEscalationAuditStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_escalation_audit("code-459", "title-986");
        item.status = RollingEscalationAuditStatus::Closed;
        assert!(
            item.move_to(RollingEscalationAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
