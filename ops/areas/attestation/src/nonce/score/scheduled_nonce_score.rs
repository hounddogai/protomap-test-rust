use super::*;

use acme_kit::text;

/// Where a scheduled nonce score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledNonceScoreStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledNonceScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ScheduledNonceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled nonce score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceScore {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub description: String,
    pub owner_id: String,
    pub status: ScheduledNonceScoreStatus,
}

impl ScheduledNonceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledNonceScoreStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledNonceScoreStatus) -> bool {
        match self.status {
            ScheduledNonceScoreStatus::New => {
                match next {
                    ScheduledNonceScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledNonceScoreStatus::Queued => {
                match next {
                    ScheduledNonceScoreStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledNonceScoreStatus::Running => {
                return next == ScheduledNonceScoreStatus::Done || next == ScheduledNonceScoreStatus::Failed;
            }
            ScheduledNonceScoreStatus::Done => {
                return next == ScheduledNonceScoreStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledNonceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled nonce score {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> ScheduledNonceScore {
        ScheduledNonceScore {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled nonce score in status new.
pub fn new_scheduled_nonce_score(code: &str, name: &str) -> ScheduledNonceScore {
    ScheduledNonceScore {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        description: "".to_string(),
        owner_id: "".to_string(),
        status: ScheduledNonceScoreStatus::New,
    }
}
