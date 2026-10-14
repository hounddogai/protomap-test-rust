use super::*;

use acme_kit::text;

/// Where a tenant tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TenantTierStatus {
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

impl std::fmt::Display for TenantTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant tier as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantTier {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub updated_at: i64,
    pub archived: bool,
    pub pinned: bool,
    pub issuer: String,
    pub quota_units: i64,
    pub segment: String,
    pub host: String,
    pub maximum: f64,
    pub status: TenantTierStatus,
}

impl TenantTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantTierStatus) -> bool {
        match self.status {
            TenantTierStatus::Draft => {
                return next == TenantTierStatus::Pending || next == TenantTierStatus::Closed;
            }
            TenantTierStatus::Pending => {
                match next {
                    TenantTierStatus::Suspended | TenantTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantTierStatus::Active => {
                match next {
                    TenantTierStatus::Suspended | TenantTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantTierStatus::Suspended => {
                return next == TenantTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.updated_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantTier {
        TenantTier {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            updated_at: self.updated_at,
            archived: self.archived,
            pinned: self.pinned,
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            host: self.host.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tenant tier in status draft.
pub fn new_tenant_tier(code: &str, name: &str) -> TenantTier {
    TenantTier {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        updated_at: 0,
        archived: false,
        pinned: false,
        issuer: "".to_string(),
        quota_units: 0,
        segment: "".to_string(),
        host: "".to_string(),
        maximum: 0.0,
        status: TenantTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tenant_tier("code-193", "name-539");
        assert_eq!(item.status, TenantTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_tenant_tier("code-193", "name-539");
        if item.move_to(TenantTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tenant_tier("code-193", "name-539");
        item.status = TenantTierStatus::Closed;
        assert!(item.move_to(TenantTierStatus::Draft).is_err(), "expected move to fail");
    }
}
