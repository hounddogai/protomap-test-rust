use super::*;

use acme_kit::text;

/// Where a regional overage snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalOverageSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RegionalOverageSnapshotStatus {
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

impl std::fmt::Display for RegionalOverageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional overage snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalOverageSnapshot {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub burst: i64,
    pub cipher: String,
    pub discount: f64,
    pub summary: String,
    pub archived: bool,
    pub status: RegionalOverageSnapshotStatus,
}

impl RegionalOverageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalOverageSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalOverageSnapshotStatus) -> bool {
        match self.status {
            RegionalOverageSnapshotStatus::New => {
                match next {
                    RegionalOverageSnapshotStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalOverageSnapshotStatus::Queued => {
                return next == RegionalOverageSnapshotStatus::Running || next == RegionalOverageSnapshotStatus::Done;
            }
            RegionalOverageSnapshotStatus::Running => {
                match next {
                    RegionalOverageSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalOverageSnapshotStatus::Done => {
                return next == RegionalOverageSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalOverageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional overage snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional overage snapshot", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.burst as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalOverageSnapshot {
        RegionalOverageSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            burst: self.burst,
            cipher: self.cipher.clone(),
            discount: self.discount,
            summary: self.summary.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional overage snapshot in status new.
pub fn new_regional_overage_snapshot(code: &str, label: &str) -> RegionalOverageSnapshot {
    RegionalOverageSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        burst: 0,
        cipher: "".to_string(),
        discount: 0.0,
        summary: "".to_string(),
        archived: false,
        status: RegionalOverageSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_regional_overage_snapshot("code-608", "label-863");
        assert_eq!(item.status, RegionalOverageSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_regional_overage_snapshot("code-608", "label-863");
        if item.move_to(RegionalOverageSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalOverageSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_regional_overage_snapshot("code-608", "label-863");
        item.status = RegionalOverageSnapshotStatus::Failed;
        assert!(
            item.move_to(RegionalOverageSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
