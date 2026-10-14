use super::*;

use acme_kit::text;

/// Where a strict allowlist snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAllowlistSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictAllowlistSnapshotStatus {
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

impl std::fmt::Display for StrictAllowlistSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict allowlist snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistSnapshot {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub amount: f64,
    pub visible: bool,
    pub owner_id: String,
    pub cost: f64,
    pub discount: f64,
    pub ratio: f64,
    pub status: StrictAllowlistSnapshotStatus,
}

impl StrictAllowlistSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAllowlistSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAllowlistSnapshotStatus) -> bool {
        match self.status {
            StrictAllowlistSnapshotStatus::Idle => {
                match next {
                    StrictAllowlistSnapshotStatus::Ready | StrictAllowlistSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAllowlistSnapshotStatus::Warming => {
                return next == StrictAllowlistSnapshotStatus::Draining
                    || next == StrictAllowlistSnapshotStatus::Stopped;
            }
            StrictAllowlistSnapshotStatus::Ready => {
                return next == StrictAllowlistSnapshotStatus::Stopped;
            }
            StrictAllowlistSnapshotStatus::Draining => {
                match next {
                    StrictAllowlistSnapshotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictAllowlistSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict allowlist snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.discount - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictAllowlistSnapshot {
        StrictAllowlistSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            amount: self.amount,
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            cost: self.cost,
            discount: self.discount,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict allowlist snapshot in status idle.
pub fn new_strict_allowlist_snapshot(code: &str, label: &str) -> StrictAllowlistSnapshot {
    StrictAllowlistSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        amount: 0.0,
        visible: false,
        owner_id: "".to_string(),
        cost: 0.0,
        discount: 0.0,
        ratio: 0.0,
        status: StrictAllowlistSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_allowlist_snapshot("code-187", "label-76");
        assert_eq!(item.status, StrictAllowlistSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_strict_allowlist_snapshot("code-187", "label-76");
        if item.move_to(StrictAllowlistSnapshotStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAllowlistSnapshotStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_allowlist_snapshot("code-187", "label-76");
        item.status = StrictAllowlistSnapshotStatus::Stopped;
        assert!(
            item.move_to(StrictAllowlistSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
