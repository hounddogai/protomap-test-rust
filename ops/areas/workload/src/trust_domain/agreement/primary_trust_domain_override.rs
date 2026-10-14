use super::*;

use acme_kit::text;

/// Where a primary trust domain override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTrustDomainOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryTrustDomainOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PrimaryTrustDomainOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary trust domain override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainOverride {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub audience: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub port: i64,
    pub quantity: i64,
    pub region: String,
    pub starts_at: i64,
    pub namespace: String,
    pub status: PrimaryTrustDomainOverrideStatus,
}

impl PrimaryTrustDomainOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTrustDomainOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTrustDomainOverrideStatus) -> bool {
        match self.status {
            PrimaryTrustDomainOverrideStatus::Idle => {
                return next == PrimaryTrustDomainOverrideStatus::Stopped;
            }
            PrimaryTrustDomainOverrideStatus::Warming => {
                match next {
                    PrimaryTrustDomainOverrideStatus::Ready | PrimaryTrustDomainOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTrustDomainOverrideStatus::Ready => {
                match next {
                    PrimaryTrustDomainOverrideStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTrustDomainOverrideStatus::Draining => {
                match next {
                    PrimaryTrustDomainOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryTrustDomainOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary trust domain override {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.attempts as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTrustDomainOverride {
        PrimaryTrustDomainOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            audience: self.audience.clone(),
            tenant_id: self.tenant_id.clone(),
            owner_id: self.owner_id.clone(),
            port: self.port,
            quantity: self.quantity,
            region: self.region.clone(),
            starts_at: self.starts_at,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary trust domain override in status idle.
pub fn new_primary_trust_domain_override(code: &str, label: &str) -> PrimaryTrustDomainOverride {
    PrimaryTrustDomainOverride {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        audience: "".to_string(),
        tenant_id: "".to_string(),
        owner_id: "".to_string(),
        port: 0,
        quantity: 0,
        region: "".to_string(),
        starts_at: 0,
        namespace: "".to_string(),
        status: PrimaryTrustDomainOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_trust_domain_override("code-34", "label-861");
        assert_eq!(item.status, PrimaryTrustDomainOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_primary_trust_domain_override("code-34", "label-861");
        if item.move_to(PrimaryTrustDomainOverrideStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTrustDomainOverrideStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_trust_domain_override("code-34", "label-861");
        item.status = PrimaryTrustDomainOverrideStatus::Stopped;
        assert!(
            item.move_to(PrimaryTrustDomainOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
