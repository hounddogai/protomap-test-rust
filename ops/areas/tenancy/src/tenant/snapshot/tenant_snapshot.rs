use super::*;

use acme_kit::text;

/// Where a tenant snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TenantSnapshotStatus {
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

impl std::fmt::Display for TenantSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSnapshot {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub source: String,
    pub priority: i64,
    pub locale: String,
    pub limit: i64,
    pub starts_at: i64,
    pub audience: String,
    pub status: TenantSnapshotStatus,
}

impl TenantSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantSnapshotStatus) -> bool {
        match self.status {
            TenantSnapshotStatus::Draft => {
                match next {
                    TenantSnapshotStatus::Pending | TenantSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSnapshotStatus::Pending => {
                return next == TenantSnapshotStatus::Closed;
            }
            TenantSnapshotStatus::Active => {
                match next {
                    TenantSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantSnapshotStatus::Suspended => {
                match next {
                    TenantSnapshotStatus::Closed => {
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

    pub fn move_to(&mut self, next: TenantSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant snapshot {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.limit as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantSnapshot {
        TenantSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            priority: self.priority,
            locale: self.locale.clone(),
            limit: self.limit,
            starts_at: self.starts_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tenant snapshot in status draft.
pub fn new_tenant_snapshot(code: &str, name: &str) -> TenantSnapshot {
    TenantSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        source: "".to_string(),
        priority: 0,
        locale: "".to_string(),
        limit: 0,
        starts_at: 0,
        audience: "".to_string(),
        status: TenantSnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tenant_snapshot("code-850", "name-442");
        assert_eq!(item.status, TenantSnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tenant_snapshot("code-850", "name-442");
        if item.move_to(TenantSnapshotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantSnapshotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tenant_snapshot("code-850", "name-442");
        item.status = TenantSnapshotStatus::Closed;
        assert!(
            item.move_to(TenantSnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
