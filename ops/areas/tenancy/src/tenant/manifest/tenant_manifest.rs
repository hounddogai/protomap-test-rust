use super::*;

use acme_kit::text;

/// Where a tenant manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TenantManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TenantManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantManifest {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub summary: String,
    pub category: String,
    pub quantity: i64,
    pub status: TenantManifestStatus,
}

impl TenantManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantManifestStatus) -> bool {
        match self.status {
            TenantManifestStatus::Proposed => {
                return next == TenantManifestStatus::Rejected || next == TenantManifestStatus::Withdrawn;
            }
            TenantManifestStatus::Approved => {
                return next == TenantManifestStatus::Rejected || next == TenantManifestStatus::Withdrawn;
            }
            TenantManifestStatus::Rejected => {
                match next {
                    TenantManifestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TenantManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> TenantManifest {
        TenantManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            summary: self.summary.clone(),
            category: self.category.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a tenant manifest in status proposed.
pub fn new_tenant_manifest(code: &str, title: &str) -> TenantManifest {
    TenantManifest {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        summary: "".to_string(),
        category: "".to_string(),
        quantity: 0,
        status: TenantManifestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tenant_manifest("code-682", "title-488");
        assert_eq!(item.status, TenantManifestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tenant_manifest("code-682", "title-488");
        if item.move_to(TenantManifestStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantManifestStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tenant_manifest("code-682", "title-488");
        item.status = TenantManifestStatus::Withdrawn;
        assert!(
            item.move_to(TenantManifestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
