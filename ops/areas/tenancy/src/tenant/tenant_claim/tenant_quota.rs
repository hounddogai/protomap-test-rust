use super::*;

use acme_kit::text;

/// Where a tenant quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TenantQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TenantQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant quota as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantQuota {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub cipher: String,
    pub weight: f64,
    pub country: String,
    pub amount: f64,
    pub capacity: i64,
    pub port: i64,
    pub budget: f64,
    pub retries: i64,
    pub status: TenantQuotaStatus,
}

impl TenantQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantQuotaStatus) -> bool {
        match self.status {
            TenantQuotaStatus::Proposed => {
                return next == TenantQuotaStatus::Rejected || next == TenantQuotaStatus::Withdrawn;
            }
            TenantQuotaStatus::Approved => {
                return next == TenantQuotaStatus::Withdrawn;
            }
            TenantQuotaStatus::Rejected => {
                match next {
                    TenantQuotaStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TenantQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.capacity as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantQuota {
        TenantQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            cipher: self.cipher.clone(),
            weight: self.weight,
            country: self.country.clone(),
            amount: self.amount,
            capacity: self.capacity,
            port: self.port,
            budget: self.budget,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tenant quota in status proposed.
pub fn new_tenant_quota(code: &str, label: &str) -> TenantQuota {
    TenantQuota {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        cipher: "".to_string(),
        weight: 0.0,
        country: "".to_string(),
        amount: 0.0,
        capacity: 0,
        port: 0,
        budget: 0.0,
        retries: 0,
        status: TenantQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tenant_quota("code-804", "label-620");
        assert_eq!(item.status, TenantQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_tenant_quota("code-804", "label-620");
        if item.move_to(TenantQuotaStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantQuotaStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tenant_quota("code-804", "label-620");
        item.status = TenantQuotaStatus::Withdrawn;
        assert!(
            item.move_to(TenantQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
