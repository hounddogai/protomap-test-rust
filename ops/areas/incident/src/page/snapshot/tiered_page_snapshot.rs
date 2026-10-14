use super::*;

use acme_kit::text;

/// Where a tiered page snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPageSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredPageSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredPageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered page snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageSnapshot {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub rate: f64,
    pub host: String,
    pub rotation_days: i64,
    pub sequence: i64,
    pub status: TieredPageSnapshotStatus,
}

impl TieredPageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPageSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPageSnapshotStatus) -> bool {
        match self.status {
            TieredPageSnapshotStatus::Proposed => {
                return next == TieredPageSnapshotStatus::Approved || next == TieredPageSnapshotStatus::Rejected;
            }
            TieredPageSnapshotStatus::Approved => {
                match next {
                    TieredPageSnapshotStatus::Rejected | TieredPageSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPageSnapshotStatus::Rejected => {
                return next == TieredPageSnapshotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredPageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered page snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered page snapshot", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.sequence as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPageSnapshot {
        TieredPageSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            rate: self.rate,
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered page snapshot in status proposed.
pub fn new_tiered_page_snapshot(code: &str, title: &str) -> TieredPageSnapshot {
    TieredPageSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        rate: 0.0,
        host: "".to_string(),
        rotation_days: 0,
        sequence: 0,
        status: TieredPageSnapshotStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_page_snapshot("code-400", "title-393");
        assert_eq!(item.status, TieredPageSnapshotStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_page_snapshot("code-400", "title-393");
        if item.move_to(TieredPageSnapshotStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPageSnapshotStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_page_snapshot("code-400", "title-393");
        item.status = TieredPageSnapshotStatus::Withdrawn;
        assert!(
            item.move_to(TieredPageSnapshotStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
