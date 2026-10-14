use super::*;

use acme_kit::text;

/// Where a redaction snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RedactionSnapshotStatus {
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

impl std::fmt::Display for RedactionSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSnapshot {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub status: RedactionSnapshotStatus,
}

impl RedactionSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionSnapshotStatus) -> bool {
        match self.status {
            RedactionSnapshotStatus::New => {
                match next {
                    RedactionSnapshotStatus::Running | RedactionSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionSnapshotStatus::Queued => {
                return next == RedactionSnapshotStatus::Running || next == RedactionSnapshotStatus::Failed;
            }
            RedactionSnapshotStatus::Running => {
                match next {
                    RedactionSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionSnapshotStatus::Done => {
                match next {
                    RedactionSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: RedactionSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionSnapshot {
        RedactionSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a redaction snapshot in status new.
pub fn new_redaction_snapshot(code: &str, title: &str) -> RedactionSnapshot {
    RedactionSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        ttl_seconds: 0,
        external_ref: "".to_string(),
        status: RedactionSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_redaction_snapshot("code-902", "title-233");
        assert_eq!(item.status, RedactionSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_redaction_snapshot("code-902", "title-233");
        if item.move_to(RedactionSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_redaction_snapshot("code-902", "title-233");
        item.status = RedactionSnapshotStatus::Failed;
        assert!(
            item.move_to(RedactionSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
