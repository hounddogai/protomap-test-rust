use super::*;

use acme_kit::text;

/// Where a global isolation score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIsolationScoreStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalIsolationScoreStatus {
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

impl std::fmt::Display for GlobalIsolationScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global isolation score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationScore {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub created_at: i64,
    pub fingerprint: String,
    pub status: GlobalIsolationScoreStatus,
}

impl GlobalIsolationScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIsolationScoreStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIsolationScoreStatus) -> bool {
        match self.status {
            GlobalIsolationScoreStatus::Draft => {
                match next {
                    GlobalIsolationScoreStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIsolationScoreStatus::Pending => {
                return next == GlobalIsolationScoreStatus::Suspended;
            }
            GlobalIsolationScoreStatus::Active => {
                return next == GlobalIsolationScoreStatus::Suspended || next == GlobalIsolationScoreStatus::Closed;
            }
            GlobalIsolationScoreStatus::Suspended => {
                match next {
                    GlobalIsolationScoreStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalIsolationScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global isolation score {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> GlobalIsolationScore {
        GlobalIsolationScore {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a global isolation score in status draft.
pub fn new_global_isolation_score(code: &str, name: &str) -> GlobalIsolationScore {
    GlobalIsolationScore {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        created_at: 0,
        fingerprint: "".to_string(),
        status: GlobalIsolationScoreStatus::Draft,
    }
}
