use super::*;

use acme_kit::text;

/// Where a trust domain bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TrustDomainBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TrustDomainBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain bundle as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBundle {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub visible: bool,
    pub rotation_days: i64,
    pub name: String,
    pub batch_size: i64,
    pub ends_at: i64,
    pub status: TrustDomainBundleStatus,
}

impl TrustDomainBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainBundleStatus) -> bool {
        match self.status {
            TrustDomainBundleStatus::Proposed => {
                return next == TrustDomainBundleStatus::Rejected;
            }
            TrustDomainBundleStatus::Approved => {
                return next == TrustDomainBundleStatus::Withdrawn;
            }
            TrustDomainBundleStatus::Rejected => {
                match next {
                    TrustDomainBundleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TrustDomainBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain bundle {} cannot move from {} to {}",
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
        format!("{} {}", "trust domain bundle", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.rotation_days as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainBundle {
        TrustDomainBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            visible: self.visible,
            rotation_days: self.rotation_days,
            name: self.name.clone(),
            batch_size: self.batch_size,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain bundle in status proposed.
pub fn new_trust_domain_bundle(code: &str, label: &str) -> TrustDomainBundle {
    TrustDomainBundle {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        visible: false,
        rotation_days: 0,
        name: "".to_string(),
        batch_size: 0,
        ends_at: 0,
        status: TrustDomainBundleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_trust_domain_bundle("code-843", "label-986");
        assert_eq!(item.status, TrustDomainBundleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_trust_domain_bundle("code-843", "label-986");
        if item.move_to(TrustDomainBundleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainBundleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_trust_domain_bundle("code-843", "label-986");
        item.status = TrustDomainBundleStatus::Withdrawn;
        assert!(
            item.move_to(TrustDomainBundleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
