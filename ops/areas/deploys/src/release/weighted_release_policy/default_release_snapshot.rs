use super::*;

use acme_kit::text;

/// Where a default release snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReleaseSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultReleaseSnapshotStatus {
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

impl std::fmt::Display for DefaultReleaseSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default release snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseSnapshot {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub pinned: bool,
    pub maximum: f64,
    pub margin: f64,
    pub capacity: i64,
    pub visible: bool,
    pub cipher: String,
    pub ratio: f64,
    pub threshold: f64,
    pub status: DefaultReleaseSnapshotStatus,
}

impl DefaultReleaseSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReleaseSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReleaseSnapshotStatus) -> bool {
        match self.status {
            DefaultReleaseSnapshotStatus::New => {
                return next == DefaultReleaseSnapshotStatus::Queued || next == DefaultReleaseSnapshotStatus::Done;
            }
            DefaultReleaseSnapshotStatus::Queued => {
                match next {
                    DefaultReleaseSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseSnapshotStatus::Running => {
                match next {
                    DefaultReleaseSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseSnapshotStatus::Done => {
                return next == DefaultReleaseSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultReleaseSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default release snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReleaseSnapshot {
        DefaultReleaseSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            pinned: self.pinned,
            maximum: self.maximum,
            margin: self.margin,
            capacity: self.capacity,
            visible: self.visible,
            cipher: self.cipher.clone(),
            ratio: self.ratio,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default release snapshot in status new.
pub fn new_default_release_snapshot(code: &str, name: &str) -> DefaultReleaseSnapshot {
    DefaultReleaseSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        pinned: false,
        maximum: 0.0,
        margin: 0.0,
        capacity: 0,
        visible: false,
        cipher: "".to_string(),
        ratio: 0.0,
        threshold: 0.0,
        status: DefaultReleaseSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_release_snapshot("code-621", "name-956");
        assert_eq!(item.status, DefaultReleaseSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_default_release_snapshot("code-621", "name-956");
        if item.move_to(DefaultReleaseSnapshotStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultReleaseSnapshotStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_release_snapshot("code-621", "name-956");
        item.status = DefaultReleaseSnapshotStatus::Failed;
        assert!(
            item.move_to(DefaultReleaseSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
