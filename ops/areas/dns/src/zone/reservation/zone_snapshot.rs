use super::*;

use acme_kit::text;

/// Where a zone snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ZoneSnapshotStatus {
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

impl std::fmt::Display for ZoneSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSnapshot {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub expires_at: i64,
    pub quantity: i64,
    pub attempts: i64,
    pub retries: i64,
    pub weight: f64,
    pub namespace: String,
    pub ends_at: i64,
    pub phone: String,
    pub status: ZoneSnapshotStatus,
}

impl ZoneSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneSnapshotStatus) -> bool {
        match self.status {
            ZoneSnapshotStatus::Draft => {
                match next {
                    ZoneSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSnapshotStatus::Pending => {
                return next == ZoneSnapshotStatus::Suspended || next == ZoneSnapshotStatus::Closed;
            }
            ZoneSnapshotStatus::Active => {
                match next {
                    ZoneSnapshotStatus::Suspended | ZoneSnapshotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSnapshotStatus::Suspended => {
                match next {
                    ZoneSnapshotStatus::Closed => {
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

    pub fn move_to(&mut self, next: ZoneSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone snapshot {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneSnapshot {
        ZoneSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            expires_at: self.expires_at,
            quantity: self.quantity,
            attempts: self.attempts,
            retries: self.retries,
            weight: self.weight,
            namespace: self.namespace.clone(),
            ends_at: self.ends_at,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a zone snapshot in status draft.
pub fn new_zone_snapshot(code: &str, name: &str) -> ZoneSnapshot {
    ZoneSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        expires_at: 0,
        quantity: 0,
        attempts: 0,
        retries: 0,
        weight: 0.0,
        namespace: "".to_string(),
        ends_at: 0,
        phone: "".to_string(),
        status: ZoneSnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_zone_snapshot("code-677", "name-941");
        assert_eq!(item.status, ZoneSnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_zone_snapshot("code-677", "name-941");
        if item.move_to(ZoneSnapshotStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneSnapshotStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_zone_snapshot("code-677", "name-941");
        item.status = ZoneSnapshotStatus::Closed;
        assert!(
            item.move_to(ZoneSnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
