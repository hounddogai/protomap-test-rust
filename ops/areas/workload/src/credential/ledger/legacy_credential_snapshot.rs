use super::*;

use acme_kit::text;

/// Where a legacy credential snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyCredentialSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyCredentialSnapshotStatus {
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

impl std::fmt::Display for LegacyCredentialSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy credential snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSnapshot {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub reviewer: String,
    pub created_at: i64,
    pub status: LegacyCredentialSnapshotStatus,
}

impl LegacyCredentialSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyCredentialSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyCredentialSnapshotStatus) -> bool {
        match self.status {
            LegacyCredentialSnapshotStatus::Idle => {
                return next == LegacyCredentialSnapshotStatus::Draining
                    || next == LegacyCredentialSnapshotStatus::Stopped;
            }
            LegacyCredentialSnapshotStatus::Warming => {
                match next {
                    LegacyCredentialSnapshotStatus::Ready | LegacyCredentialSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCredentialSnapshotStatus::Ready => {
                match next {
                    LegacyCredentialSnapshotStatus::Draining | LegacyCredentialSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyCredentialSnapshotStatus::Draining => {
                return next == LegacyCredentialSnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyCredentialSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy credential snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.created_at as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyCredentialSnapshot {
        LegacyCredentialSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            reviewer: self.reviewer.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy credential snapshot in status idle.
pub fn new_legacy_credential_snapshot(code: &str, name: &str) -> LegacyCredentialSnapshot {
    LegacyCredentialSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        reviewer: "".to_string(),
        created_at: 0,
        status: LegacyCredentialSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_credential_snapshot("code-350", "name-86");
        assert_eq!(item.status, LegacyCredentialSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_legacy_credential_snapshot("code-350", "name-86");
        if item.move_to(LegacyCredentialSnapshotStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyCredentialSnapshotStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_credential_snapshot("code-350", "name-86");
        item.status = LegacyCredentialSnapshotStatus::Stopped;
        assert!(
            item.move_to(LegacyCredentialSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
