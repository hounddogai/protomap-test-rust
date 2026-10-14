use super::*;

use acme_kit::text;

/// Where a batched tenant bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTenantBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedTenantBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedTenantBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched tenant bundle as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantBundle {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub region: String,
    pub description: String,
    pub capacity: i64,
    pub currency: String,
    pub threshold: f64,
    pub floor_value: f64,
    pub status: BatchedTenantBundleStatus,
}

impl BatchedTenantBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTenantBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTenantBundleStatus) -> bool {
        match self.status {
            BatchedTenantBundleStatus::Proposed => {
                return next == BatchedTenantBundleStatus::Rejected;
            }
            BatchedTenantBundleStatus::Approved => {
                match next {
                    BatchedTenantBundleStatus::Rejected | BatchedTenantBundleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTenantBundleStatus::Rejected => {
                return next == BatchedTenantBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTenantBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched tenant bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.capacity as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTenantBundle {
        BatchedTenantBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            region: self.region.clone(),
            description: self.description.clone(),
            capacity: self.capacity,
            currency: self.currency.clone(),
            threshold: self.threshold,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched tenant bundle in status proposed.
pub fn new_batched_tenant_bundle(code: &str, name: &str) -> BatchedTenantBundle {
    BatchedTenantBundle {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        region: "".to_string(),
        description: "".to_string(),
        capacity: 0,
        currency: "".to_string(),
        threshold: 0.0,
        floor_value: 0.0,
        status: BatchedTenantBundleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_tenant_bundle("code-820", "name-534");
        assert_eq!(item.status, BatchedTenantBundleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_batched_tenant_bundle("code-820", "name-534");
        if item.move_to(BatchedTenantBundleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTenantBundleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_tenant_bundle("code-820", "name-534");
        item.status = BatchedTenantBundleStatus::Withdrawn;
        assert!(
            item.move_to(BatchedTenantBundleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
