use super::*;

use acme_kit::text;

/// Where a legacy rollback milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRollbackMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyRollbackMilestoneStatus {
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

impl std::fmt::Display for LegacyRollbackMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rollback milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMilestone {
    pub code: String,
    pub name: String,
    pub title: String,
    pub phone: String,
    pub summary: String,
    pub cipher: String,
    pub port: i64,
    pub fingerprint: String,
    pub created_at: i64,
    pub status: LegacyRollbackMilestoneStatus,
}

impl LegacyRollbackMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRollbackMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRollbackMilestoneStatus) -> bool {
        match self.status {
            LegacyRollbackMilestoneStatus::New => {
                match next {
                    LegacyRollbackMilestoneStatus::Queued | LegacyRollbackMilestoneStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackMilestoneStatus::Queued => {
                match next {
                    LegacyRollbackMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackMilestoneStatus::Running => {
                match next {
                    LegacyRollbackMilestoneStatus::Done | LegacyRollbackMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackMilestoneStatus::Done => {
                match next {
                    LegacyRollbackMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyRollbackMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollback milestone {} cannot move from {} to {}",
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
        format!("{} {}", "legacy rollback milestone", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.port as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRollbackMilestone {
        LegacyRollbackMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            summary: self.summary.clone(),
            cipher: self.cipher.clone(),
            port: self.port,
            fingerprint: self.fingerprint.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy rollback milestone in status new.
pub fn new_legacy_rollback_milestone(code: &str, name: &str) -> LegacyRollbackMilestone {
    LegacyRollbackMilestone {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        phone: "".to_string(),
        summary: "".to_string(),
        cipher: "".to_string(),
        port: 0,
        fingerprint: "".to_string(),
        created_at: 0,
        status: LegacyRollbackMilestoneStatus::New,
    }
}
