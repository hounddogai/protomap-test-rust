use super::*;

use acme_kit::text;

/// Where a canary snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanarySnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CanarySnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CanarySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshot {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub country: String,
    pub locale: String,
    pub created_at: i64,
    pub fingerprint: String,
    pub checksum: String,
    pub name: String,
    pub status: CanarySnapshotStatus,
}

impl CanarySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanarySnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanarySnapshotStatus) -> bool {
        match self.status {
            CanarySnapshotStatus::Draft => {
                return next == CanarySnapshotStatus::Active || next == CanarySnapshotStatus::Suspended;
            }
            CanarySnapshotStatus::Pending => {
                return next == CanarySnapshotStatus::Closed;
            }
            CanarySnapshotStatus::Active => {
                match next {
                    CanarySnapshotStatus::Suspended | CanarySnapshotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanarySnapshotStatus::Suspended => {
                match next {
                    CanarySnapshotStatus::Closed => {
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

    pub fn move_to(&mut self, next: CanarySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary snapshot {} cannot move from {} to {}",
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
        format!("{} {}", "canary snapshot", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ratio - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanarySnapshot {
        CanarySnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            country: self.country.clone(),
            locale: self.locale.clone(),
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a canary snapshot in status draft.
pub fn new_canary_snapshot(code: &str, title: &str) -> CanarySnapshot {
    CanarySnapshot {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        country: "".to_string(),
        locale: "".to_string(),
        created_at: 0,
        fingerprint: "".to_string(),
        checksum: "".to_string(),
        name: "".to_string(),
        status: CanarySnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_canary_snapshot("code-873", "title-816");
        assert_eq!(item.status, CanarySnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_canary_snapshot("code-873", "title-816");
        if item.move_to(CanarySnapshotStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanarySnapshotStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_canary_snapshot("code-873", "title-816");
        item.status = CanarySnapshotStatus::Closed;
        assert!(
            item.move_to(CanarySnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
