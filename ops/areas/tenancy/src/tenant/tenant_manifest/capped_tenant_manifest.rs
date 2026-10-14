use super::*;

use acme_kit::text;

/// Where a capped tenant manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTenantManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedTenantManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedTenantManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped tenant manifest as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantManifest {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub grace_days: i64,
    pub active: bool,
    pub principal: String,
    pub approved: bool,
    pub currency: String,
    pub status: CappedTenantManifestStatus,
}

impl CappedTenantManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTenantManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTenantManifestStatus) -> bool {
        match self.status {
            CappedTenantManifestStatus::Proposed => {
                match next {
                    CappedTenantManifestStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTenantManifestStatus::Approved => {
                return next == CappedTenantManifestStatus::Rejected || next == CappedTenantManifestStatus::Withdrawn;
            }
            CappedTenantManifestStatus::Rejected => {
                return next == CappedTenantManifestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTenantManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped tenant manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> CappedTenantManifest {
        CappedTenantManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            grace_days: self.grace_days,
            active: self.active,
            principal: self.principal.clone(),
            approved: self.approved,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped tenant manifest in status proposed.
pub fn new_capped_tenant_manifest(code: &str, title: &str) -> CappedTenantManifest {
    CappedTenantManifest {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        grace_days: 0,
        active: false,
        principal: "".to_string(),
        approved: false,
        currency: "".to_string(),
        status: CappedTenantManifestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_tenant_manifest("code-338", "title-558");
        assert_eq!(item.status, CappedTenantManifestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_capped_tenant_manifest("code-338", "title-558");
        if item.move_to(CappedTenantManifestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTenantManifestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_tenant_manifest("code-338", "title-558");
        item.status = CappedTenantManifestStatus::Withdrawn;
        assert!(
            item.move_to(CappedTenantManifestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
