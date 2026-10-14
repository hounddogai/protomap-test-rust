use super::*;

use acme_kit::text;

/// Where a trust domain adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TrustDomainAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TrustDomainAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAdjustment {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub minimum: f64,
    pub expires_at: i64,
    pub pinned: bool,
    pub status: TrustDomainAdjustmentStatus,
}

impl TrustDomainAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainAdjustmentStatus) -> bool {
        match self.status {
            TrustDomainAdjustmentStatus::Proposed => {
                match next {
                    TrustDomainAdjustmentStatus::Approved | TrustDomainAdjustmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainAdjustmentStatus::Approved => {
                return next == TrustDomainAdjustmentStatus::Withdrawn;
            }
            TrustDomainAdjustmentStatus::Rejected => {
                return next == TrustDomainAdjustmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain adjustment {} cannot move from {} to {}",
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
        format!("{} {}", "trust domain adjustment", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.minimum - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainAdjustment {
        TrustDomainAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            minimum: self.minimum,
            expires_at: self.expires_at,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a trust domain adjustment in status proposed.
pub fn new_trust_domain_adjustment(code: &str, name: &str) -> TrustDomainAdjustment {
    TrustDomainAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        minimum: 0.0,
        expires_at: 0,
        pinned: false,
        status: TrustDomainAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_trust_domain_adjustment("code-491", "name-479");
        assert_eq!(item.status, TrustDomainAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_trust_domain_adjustment("code-491", "name-479");
        if item.move_to(TrustDomainAdjustmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainAdjustmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_trust_domain_adjustment("code-491", "name-479");
        item.status = TrustDomainAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(TrustDomainAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
