use super::*;

use acme_kit::text;

/// Where a probe snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe snapshot as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshot {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub priority: i64,
    pub expires_at: i64,
    pub key_id: String,
    pub created_at: i64,
    pub limit: i64,
    pub visible: bool,
    pub status: ProbeSnapshotStatus,
}

impl ProbeSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeSnapshotStatus) -> bool {
        match self.status {
            ProbeSnapshotStatus::Created => {
                return next == ProbeSnapshotStatus::Retired;
            }
            ProbeSnapshotStatus::Validated => {
                return next == ProbeSnapshotStatus::Retired;
            }
            ProbeSnapshotStatus::Published => {
                match next {
                    ProbeSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: ProbeSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe snapshot", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.expires_at as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeSnapshot {
        ProbeSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            priority: self.priority,
            expires_at: self.expires_at,
            key_id: self.key_id.clone(),
            created_at: self.created_at,
            limit: self.limit,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a probe snapshot in status created.
pub fn new_probe_snapshot(code: &str, name: &str) -> ProbeSnapshot {
    ProbeSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        priority: 0,
        expires_at: 0,
        key_id: "".to_string(),
        created_at: 0,
        limit: 0,
        visible: false,
        status: ProbeSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_probe_snapshot("code-966", "name-217");
        assert_eq!(item.status, ProbeSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_probe_snapshot("code-966", "name-217");
        if item.move_to(ProbeSnapshotStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeSnapshotStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_probe_snapshot("code-966", "name-217");
        item.status = ProbeSnapshotStatus::Retired;
        assert!(
            item.move_to(ProbeSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
