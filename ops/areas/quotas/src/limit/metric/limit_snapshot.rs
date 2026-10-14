use super::*;

use acme_kit::text;

/// Where a limit snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LimitSnapshotStatus {
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

impl std::fmt::Display for LimitSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitSnapshot {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub visible: bool,
    pub quantity: i64,
    pub rate: f64,
    pub name: String,
    pub version: i64,
    pub starts_at: i64,
    pub burst: i64,
    pub locale: String,
    pub status: LimitSnapshotStatus,
}

impl LimitSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitSnapshotStatus) -> bool {
        match self.status {
            LimitSnapshotStatus::New => {
                match next {
                    LimitSnapshotStatus::Running | LimitSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitSnapshotStatus::Queued => {
                return next == LimitSnapshotStatus::Done;
            }
            LimitSnapshotStatus::Running => {
                match next {
                    LimitSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitSnapshotStatus::Done => {
                match next {
                    LimitSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: LimitSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "limit snapshot", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.version as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitSnapshot {
        LimitSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            visible: self.visible,
            quantity: self.quantity,
            rate: self.rate,
            name: self.name.clone(),
            version: self.version,
            starts_at: self.starts_at,
            burst: self.burst,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a limit snapshot in status new.
pub fn new_limit_snapshot(code: &str, label: &str) -> LimitSnapshot {
    LimitSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        visible: false,
        quantity: 0,
        rate: 0.0,
        name: "".to_string(),
        version: 0,
        starts_at: 0,
        burst: 0,
        locale: "".to_string(),
        status: LimitSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_limit_snapshot("code-300", "label-780");
        assert_eq!(item.status, LimitSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_limit_snapshot("code-300", "label-780");
        if item.move_to(LimitSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_limit_snapshot("code-300", "label-780");
        item.status = LimitSnapshotStatus::Failed;
        assert!(item.move_to(LimitSnapshotStatus::New).is_err(), "expected move to fail");
    }
}
