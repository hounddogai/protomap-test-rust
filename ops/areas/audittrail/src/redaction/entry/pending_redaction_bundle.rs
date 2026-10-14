use super::*;

use acme_kit::text;

/// Where a pending redaction bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRedactionBundleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingRedactionBundleStatus {
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

impl std::fmt::Display for PendingRedactionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending redaction bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionBundle {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub burst: i64,
    pub floor_value: f64,
    pub expires_at: i64,
    pub cluster: String,
    pub source: String,
    pub rotation_days: i64,
    pub status: PendingRedactionBundleStatus,
}

impl PendingRedactionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRedactionBundleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRedactionBundleStatus) -> bool {
        match self.status {
            PendingRedactionBundleStatus::New => {
                return next == PendingRedactionBundleStatus::Failed;
            }
            PendingRedactionBundleStatus::Queued => {
                return next == PendingRedactionBundleStatus::Done || next == PendingRedactionBundleStatus::Failed;
            }
            PendingRedactionBundleStatus::Running => {
                match next {
                    PendingRedactionBundleStatus::Done | PendingRedactionBundleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRedactionBundleStatus::Done => {
                return next == PendingRedactionBundleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRedactionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending redaction bundle {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.floor_value - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRedactionBundle {
        PendingRedactionBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            burst: self.burst,
            floor_value: self.floor_value,
            expires_at: self.expires_at,
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending redaction bundle in status new.
pub fn new_pending_redaction_bundle(code: &str, title: &str) -> PendingRedactionBundle {
    PendingRedactionBundle {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        burst: 0,
        floor_value: 0.0,
        expires_at: 0,
        cluster: "".to_string(),
        source: "".to_string(),
        rotation_days: 0,
        status: PendingRedactionBundleStatus::New,
    }
}
