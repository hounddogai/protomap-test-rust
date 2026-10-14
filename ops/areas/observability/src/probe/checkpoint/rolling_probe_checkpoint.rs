use super::*;

use acme_kit::text;

/// Where a rolling probe checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingProbeCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingProbeCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingProbeCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling probe checkpoint as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpoint {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub pinned: bool,
    pub starts_at: i64,
    pub status: RollingProbeCheckpointStatus,
}

impl RollingProbeCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingProbeCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingProbeCheckpointStatus) -> bool {
        match self.status {
            RollingProbeCheckpointStatus::Requested => {
                match next {
                    RollingProbeCheckpointStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingProbeCheckpointStatus::Granted => {
                return next == RollingProbeCheckpointStatus::Expired;
            }
            RollingProbeCheckpointStatus::Revoked => {
                return next == RollingProbeCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingProbeCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling probe checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling probe checkpoint", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.ceiling * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingProbeCheckpoint {
        RollingProbeCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            pinned: self.pinned,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling probe checkpoint in status requested.
pub fn new_rolling_probe_checkpoint(code: &str, name: &str) -> RollingProbeCheckpoint {
    RollingProbeCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        pinned: false,
        starts_at: 0,
        status: RollingProbeCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_probe_checkpoint("code-230", "name-965");
        assert_eq!(item.status, RollingProbeCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rolling_probe_checkpoint("code-230", "name-965");
        if item.move_to(RollingProbeCheckpointStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingProbeCheckpointStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_probe_checkpoint("code-230", "name-965");
        item.status = RollingProbeCheckpointStatus::Expired;
        assert!(
            item.move_to(RollingProbeCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
