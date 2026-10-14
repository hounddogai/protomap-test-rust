use super::*;

use acme_kit::text;

/// Where a regional freeze snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFreezeSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalFreezeSnapshotStatus {
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

impl std::fmt::Display for RegionalFreezeSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional freeze snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFreezeSnapshot {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub key_id: String,
    pub threshold: f64,
    pub timezone: String,
    pub status: RegionalFreezeSnapshotStatus,
}

impl RegionalFreezeSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFreezeSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFreezeSnapshotStatus) -> bool {
        match self.status {
            RegionalFreezeSnapshotStatus::Idle => {
                return next == RegionalFreezeSnapshotStatus::Warming || next == RegionalFreezeSnapshotStatus::Stopped;
            }
            RegionalFreezeSnapshotStatus::Warming => {
                match next {
                    RegionalFreezeSnapshotStatus::Draining | RegionalFreezeSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFreezeSnapshotStatus::Ready => {
                return next == RegionalFreezeSnapshotStatus::Draining || next == RegionalFreezeSnapshotStatus::Stopped;
            }
            RegionalFreezeSnapshotStatus::Draining => {
                match next {
                    RegionalFreezeSnapshotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalFreezeSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional freeze snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional freeze snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> RegionalFreezeSnapshot {
        RegionalFreezeSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            key_id: self.key_id.clone(),
            threshold: self.threshold,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional freeze snapshot in status idle.
pub fn new_regional_freeze_snapshot(code: &str, name: &str) -> RegionalFreezeSnapshot {
    RegionalFreezeSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        key_id: "".to_string(),
        threshold: 0.0,
        timezone: "".to_string(),
        status: RegionalFreezeSnapshotStatus::Idle,
    }
}
