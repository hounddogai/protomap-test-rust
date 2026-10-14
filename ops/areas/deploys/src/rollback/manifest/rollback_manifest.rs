use super::*;

use acme_kit::text;

/// Where a rollback manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackManifestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollbackManifestStatus {
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

impl std::fmt::Display for RollbackManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackManifest {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub ends_at: i64,
    pub timezone: String,
    pub status: RollbackManifestStatus,
}

impl RollbackManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackManifestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackManifestStatus) -> bool {
        match self.status {
            RollbackManifestStatus::Idle => {
                match next {
                    RollbackManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackManifestStatus::Warming => {
                return next == RollbackManifestStatus::Draining;
            }
            RollbackManifestStatus::Ready => {
                match next {
                    RollbackManifestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackManifestStatus::Draining => {
                match next {
                    RollbackManifestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollbackManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback manifest {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> RollbackManifest {
        RollbackManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            ends_at: self.ends_at,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollback manifest in status idle.
pub fn new_rollback_manifest(code: &str, name: &str) -> RollbackManifest {
    RollbackManifest {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        ends_at: 0,
        timezone: "".to_string(),
        status: RollbackManifestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rollback_manifest("code-484", "name-884");
        assert_eq!(item.status, RollbackManifestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_rollback_manifest("code-484", "name-884");
        if item.move_to(RollbackManifestStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackManifestStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rollback_manifest("code-484", "name-884");
        item.status = RollbackManifestStatus::Stopped;
        assert!(
            item.move_to(RollbackManifestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
