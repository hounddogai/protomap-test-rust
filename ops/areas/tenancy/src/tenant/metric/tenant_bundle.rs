use super::*;

use acme_kit::text;

/// Where a tenant bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TenantBundleStatus {
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

impl std::fmt::Display for TenantBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantBundle {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub approved: bool,
    pub scope: String,
    pub status: TenantBundleStatus,
}

impl TenantBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantBundleStatus) -> bool {
        match self.status {
            TenantBundleStatus::Draft => {
                match next {
                    TenantBundleStatus::Pending | TenantBundleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantBundleStatus::Pending => {
                return next == TenantBundleStatus::Active;
            }
            TenantBundleStatus::Active => {
                match next {
                    TenantBundleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantBundleStatus::Suspended => {
                match next {
                    TenantBundleStatus::Closed => {
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

    pub fn move_to(&mut self, next: TenantBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> TenantBundle {
        TenantBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            approved: self.approved,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tenant bundle in status draft.
pub fn new_tenant_bundle(code: &str, label: &str) -> TenantBundle {
    TenantBundle {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        approved: false,
        scope: "".to_string(),
        status: TenantBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tenant_bundle("code-472", "label-542");
        assert_eq!(item.status, TenantBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tenant_bundle("code-472", "label-542");
        if item.move_to(TenantBundleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantBundleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tenant_bundle("code-472", "label-542");
        item.status = TenantBundleStatus::Closed;
        assert!(
            item.move_to(TenantBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
