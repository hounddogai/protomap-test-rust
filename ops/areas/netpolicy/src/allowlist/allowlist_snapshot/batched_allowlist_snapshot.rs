use super::*;

use acme_kit::text;

/// Where a batched allowlist snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedAllowlistSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedAllowlistSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched allowlist snapshot as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistSnapshot {
    pub code: String,
    pub label: String,
    pub region: String,
    pub pinned: bool,
    pub updated_at: i64,
    pub limit: i64,
    pub capacity: i64,
    pub ratio: f64,
    pub status: BatchedAllowlistSnapshotStatus,
}

impl BatchedAllowlistSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistSnapshotStatus) -> bool {
        match self.status {
            BatchedAllowlistSnapshotStatus::Idle => {
                return next == BatchedAllowlistSnapshotStatus::Warming
                    || next == BatchedAllowlistSnapshotStatus::Draining;
            }
            BatchedAllowlistSnapshotStatus::Warming => {
                return next == BatchedAllowlistSnapshotStatus::Draining;
            }
            BatchedAllowlistSnapshotStatus::Ready => {
                return next == BatchedAllowlistSnapshotStatus::Draining
                    || next == BatchedAllowlistSnapshotStatus::Stopped;
            }
            BatchedAllowlistSnapshotStatus::Draining => {
                match next {
                    BatchedAllowlistSnapshotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedAllowlistSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched allowlist snapshot", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.updated_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAllowlistSnapshot {
        BatchedAllowlistSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            pinned: self.pinned,
            updated_at: self.updated_at,
            limit: self.limit,
            capacity: self.capacity,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a batched allowlist snapshot in status idle.
pub fn new_batched_allowlist_snapshot(code: &str, label: &str) -> BatchedAllowlistSnapshot {
    BatchedAllowlistSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        pinned: false,
        updated_at: 0,
        limit: 0,
        capacity: 0,
        ratio: 0.0,
        status: BatchedAllowlistSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_allowlist_snapshot("code-198", "label-880");
        assert_eq!(item.status, BatchedAllowlistSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_allowlist_snapshot("code-198", "label-880");
        if item.move_to(BatchedAllowlistSnapshotStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAllowlistSnapshotStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_allowlist_snapshot("code-198", "label-880");
        item.status = BatchedAllowlistSnapshotStatus::Stopped;
        assert!(
            item.move_to(BatchedAllowlistSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
