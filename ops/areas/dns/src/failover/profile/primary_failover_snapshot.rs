use super::*;

use acme_kit::text;

/// Where a primary failover snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFailoverSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryFailoverSnapshotStatus {
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

impl std::fmt::Display for PrimaryFailoverSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary failover snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverSnapshot {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub minimum: f64,
    pub approved: bool,
    pub segment: String,
    pub updated_at: i64,
    pub locale: String,
    pub status: PrimaryFailoverSnapshotStatus,
}

impl PrimaryFailoverSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFailoverSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFailoverSnapshotStatus) -> bool {
        match self.status {
            PrimaryFailoverSnapshotStatus::Idle => {
                match next {
                    PrimaryFailoverSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverSnapshotStatus::Warming => {
                match next {
                    PrimaryFailoverSnapshotStatus::Ready | PrimaryFailoverSnapshotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverSnapshotStatus::Ready => {
                match next {
                    PrimaryFailoverSnapshotStatus::Draining | PrimaryFailoverSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverSnapshotStatus::Draining => {
                match next {
                    PrimaryFailoverSnapshotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryFailoverSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary failover snapshot {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "primary failover snapshot", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.grace_days as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFailoverSnapshot {
        PrimaryFailoverSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            minimum: self.minimum,
            approved: self.approved,
            segment: self.segment.clone(),
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary failover snapshot in status idle.
pub fn new_primary_failover_snapshot(code: &str, name: &str) -> PrimaryFailoverSnapshot {
    PrimaryFailoverSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        minimum: 0.0,
        approved: false,
        segment: "".to_string(),
        updated_at: 0,
        locale: "".to_string(),
        status: PrimaryFailoverSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_failover_snapshot("code-673", "name-550");
        assert_eq!(item.status, PrimaryFailoverSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_primary_failover_snapshot("code-673", "name-550");
        if item.move_to(PrimaryFailoverSnapshotStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryFailoverSnapshotStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_failover_snapshot("code-673", "name-550");
        item.status = PrimaryFailoverSnapshotStatus::Stopped;
        assert!(
            item.move_to(PrimaryFailoverSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
