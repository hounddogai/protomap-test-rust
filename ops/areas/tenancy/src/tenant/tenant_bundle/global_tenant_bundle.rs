use super::*;

use acme_kit::text;

/// Where a global tenant bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTenantBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalTenantBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalTenantBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global tenant bundle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantBundle {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub cluster: String,
    pub weight: f64,
    pub starts_at: i64,
    pub active: bool,
    pub batch_size: i64,
    pub status: GlobalTenantBundleStatus,
}

impl GlobalTenantBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTenantBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTenantBundleStatus) -> bool {
        match self.status {
            GlobalTenantBundleStatus::Requested => {
                match next {
                    GlobalTenantBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTenantBundleStatus::Granted => {
                return next == GlobalTenantBundleStatus::Revoked || next == GlobalTenantBundleStatus::Expired;
            }
            GlobalTenantBundleStatus::Revoked => {
                return next == GlobalTenantBundleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalTenantBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global tenant bundle {} cannot move from {} to {}",
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

    pub fn projected_value(&self) -> f64 {
        ((self.batch_size as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTenantBundle {
        GlobalTenantBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            cluster: self.cluster.clone(),
            weight: self.weight,
            starts_at: self.starts_at,
            active: self.active,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global tenant bundle in status requested.
pub fn new_global_tenant_bundle(code: &str, name: &str) -> GlobalTenantBundle {
    GlobalTenantBundle {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        cluster: "".to_string(),
        weight: 0.0,
        starts_at: 0,
        active: false,
        batch_size: 0,
        status: GlobalTenantBundleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_tenant_bundle("code-648", "name-819");
        assert_eq!(item.status, GlobalTenantBundleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_global_tenant_bundle("code-648", "name-819");
        if item.move_to(GlobalTenantBundleStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalTenantBundleStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_tenant_bundle("code-648", "name-819");
        item.status = GlobalTenantBundleStatus::Expired;
        assert!(
            item.move_to(GlobalTenantBundleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
