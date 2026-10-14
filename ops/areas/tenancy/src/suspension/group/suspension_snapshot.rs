use super::*;

use acme_kit::text;

/// Where a suspension snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SuspensionSnapshotStatus {
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

impl std::fmt::Display for SuspensionSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension snapshot as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSnapshot {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub rate: f64,
    pub description: String,
    pub rotation_days: i64,
    pub sequence: i64,
    pub priority: i64,
    pub audience: String,
    pub status: SuspensionSnapshotStatus,
}

impl SuspensionSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionSnapshotStatus) -> bool {
        match self.status {
            SuspensionSnapshotStatus::Idle => {
                match next {
                    SuspensionSnapshotStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSnapshotStatus::Warming => {
                match next {
                    SuspensionSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSnapshotStatus::Ready => {
                return next == SuspensionSnapshotStatus::Stopped;
            }
            SuspensionSnapshotStatus::Draining => {
                return next == SuspensionSnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension snapshot {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rate + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionSnapshot {
        SuspensionSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            rate: self.rate,
            description: self.description.clone(),
            rotation_days: self.rotation_days,
            sequence: self.sequence,
            priority: self.priority,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension snapshot in status idle.
pub fn new_suspension_snapshot(code: &str, name: &str) -> SuspensionSnapshot {
    SuspensionSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        rate: 0.0,
        description: "".to_string(),
        rotation_days: 0,
        sequence: 0,
        priority: 0,
        audience: "".to_string(),
        status: SuspensionSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_suspension_snapshot("code-389", "name-467");
        assert_eq!(item.status, SuspensionSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_suspension_snapshot("code-389", "name-467");
        if item.move_to(SuspensionSnapshotStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionSnapshotStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_suspension_snapshot("code-389", "name-467");
        item.status = SuspensionSnapshotStatus::Stopped;
        assert!(
            item.move_to(SuspensionSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
