use super::*;

use acme_kit::text;

/// Where a global escalation rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalEscalationRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalEscalationRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalEscalationRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global escalation rollup as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEscalationRollup {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub amount: f64,
    pub active: bool,
    pub port: i64,
    pub created_at: i64,
    pub sequence: i64,
    pub status: GlobalEscalationRollupStatus,
}

impl GlobalEscalationRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalEscalationRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalEscalationRollupStatus) -> bool {
        match self.status {
            GlobalEscalationRollupStatus::Requested => {
                match next {
                    GlobalEscalationRollupStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalEscalationRollupStatus::Granted => {
                return next == GlobalEscalationRollupStatus::Revoked || next == GlobalEscalationRollupStatus::Expired;
            }
            GlobalEscalationRollupStatus::Revoked => {
                return next == GlobalEscalationRollupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalEscalationRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global escalation rollup {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalEscalationRollup {
        GlobalEscalationRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            active: self.active,
            port: self.port,
            created_at: self.created_at,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global escalation rollup in status requested.
pub fn new_global_escalation_rollup(code: &str, title: &str) -> GlobalEscalationRollup {
    GlobalEscalationRollup {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        amount: 0.0,
        active: false,
        port: 0,
        created_at: 0,
        sequence: 0,
        status: GlobalEscalationRollupStatus::Requested,
    }
}
