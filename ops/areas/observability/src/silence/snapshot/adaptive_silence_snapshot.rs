use super::*;

use acme_kit::text;

/// Where a adaptive silence snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSilenceSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveSilenceSnapshotStatus {
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

impl std::fmt::Display for AdaptiveSilenceSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive silence snapshot as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceSnapshot {
    pub code: String,
    pub name: String,
    pub description: String,
    pub issuer: String,
    pub budget: f64,
    pub created_at: i64,
    pub batch_size: i64,
    pub namespace: String,
    pub approved: bool,
    pub owner_id: String,
    pub summary: String,
    pub status: AdaptiveSilenceSnapshotStatus,
}

impl AdaptiveSilenceSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSilenceSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSilenceSnapshotStatus) -> bool {
        match self.status {
            AdaptiveSilenceSnapshotStatus::New => {
                return next == AdaptiveSilenceSnapshotStatus::Running;
            }
            AdaptiveSilenceSnapshotStatus::Queued => {
                return next == AdaptiveSilenceSnapshotStatus::Running || next == AdaptiveSilenceSnapshotStatus::Done;
            }
            AdaptiveSilenceSnapshotStatus::Running => {
                return next == AdaptiveSilenceSnapshotStatus::Failed;
            }
            AdaptiveSilenceSnapshotStatus::Done => {
                return next == AdaptiveSilenceSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSilenceSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive silence snapshot {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.created_at as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSilenceSnapshot {
        AdaptiveSilenceSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            budget: self.budget,
            created_at: self.created_at,
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            approved: self.approved,
            owner_id: self.owner_id.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive silence snapshot in status new.
pub fn new_adaptive_silence_snapshot(code: &str, name: &str) -> AdaptiveSilenceSnapshot {
    AdaptiveSilenceSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        issuer: "".to_string(),
        budget: 0.0,
        created_at: 0,
        batch_size: 0,
        namespace: "".to_string(),
        approved: false,
        owner_id: "".to_string(),
        summary: "".to_string(),
        status: AdaptiveSilenceSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_silence_snapshot("code-615", "name-765");
        assert_eq!(item.status, AdaptiveSilenceSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_adaptive_silence_snapshot("code-615", "name-765");
        if item.move_to(AdaptiveSilenceSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSilenceSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_silence_snapshot("code-615", "name-765");
        item.status = AdaptiveSilenceSnapshotStatus::Failed;
        assert!(
            item.move_to(AdaptiveSilenceSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
