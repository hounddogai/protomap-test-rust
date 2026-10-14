use super::*;

use acme_kit::text;

/// Where a pending tenant adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTenantAdjustmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingTenantAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingTenantAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending tenant adjustment as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTenantAdjustment {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub source: String,
    pub owner_id: String,
    pub maximum: f64,
    pub batch_size: i64,
    pub cluster: String,
    pub amount: f64,
    pub ends_at: i64,
    pub status: PendingTenantAdjustmentStatus,
}

impl PendingTenantAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTenantAdjustmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTenantAdjustmentStatus) -> bool {
        match self.status {
            PendingTenantAdjustmentStatus::Requested => {
                match next {
                    PendingTenantAdjustmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTenantAdjustmentStatus::Granted => {
                return next == PendingTenantAdjustmentStatus::Expired;
            }
            PendingTenantAdjustmentStatus::Revoked => {
                return next == PendingTenantAdjustmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingTenantAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending tenant adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.amount - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTenantAdjustment {
        PendingTenantAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            source: self.source.clone(),
            owner_id: self.owner_id.clone(),
            maximum: self.maximum,
            batch_size: self.batch_size,
            cluster: self.cluster.clone(),
            amount: self.amount,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending tenant adjustment in status requested.
pub fn new_pending_tenant_adjustment(code: &str, label: &str) -> PendingTenantAdjustment {
    PendingTenantAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        source: "".to_string(),
        owner_id: "".to_string(),
        maximum: 0.0,
        batch_size: 0,
        cluster: "".to_string(),
        amount: 0.0,
        ends_at: 0,
        status: PendingTenantAdjustmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_tenant_adjustment("code-509", "label-844");
        assert_eq!(item.status, PendingTenantAdjustmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_pending_tenant_adjustment("code-509", "label-844");
        if item.move_to(PendingTenantAdjustmentStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTenantAdjustmentStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_tenant_adjustment("code-509", "label-844");
        item.status = PendingTenantAdjustmentStatus::Expired;
        assert!(
            item.move_to(PendingTenantAdjustmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
