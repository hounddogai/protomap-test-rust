use super::*;

use acme_kit::text;

/// Where a fallback trust domain allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTrustDomainAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackTrustDomainAllocationStatus {
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

impl std::fmt::Display for FallbackTrustDomainAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback trust domain allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainAllocation {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub score: f64,
    pub expires_at: i64,
    pub visible: bool,
    pub cluster: String,
    pub status: FallbackTrustDomainAllocationStatus,
}

impl FallbackTrustDomainAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTrustDomainAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTrustDomainAllocationStatus) -> bool {
        match self.status {
            FallbackTrustDomainAllocationStatus::Idle => {
                match next {
                    FallbackTrustDomainAllocationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTrustDomainAllocationStatus::Warming => {
                match next {
                    FallbackTrustDomainAllocationStatus::Ready | FallbackTrustDomainAllocationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTrustDomainAllocationStatus::Ready => {
                match next {
                    FallbackTrustDomainAllocationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTrustDomainAllocationStatus::Draining => {
                match next {
                    FallbackTrustDomainAllocationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackTrustDomainAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback trust domain allocation {} cannot move from {} to {}",
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
        format!("{} {}", "fallback trust domain allocation", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.score * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackTrustDomainAllocation {
        FallbackTrustDomainAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            score: self.score,
            expires_at: self.expires_at,
            visible: self.visible,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback trust domain allocation in status idle.
pub fn new_fallback_trust_domain_allocation(code: &str, name: &str) -> FallbackTrustDomainAllocation {
    FallbackTrustDomainAllocation {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        score: 0.0,
        expires_at: 0,
        visible: false,
        cluster: "".to_string(),
        status: FallbackTrustDomainAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_trust_domain_allocation("code-825", "name-245");
        assert_eq!(item.status, FallbackTrustDomainAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_fallback_trust_domain_allocation("code-825", "name-245");
        if item.move_to(FallbackTrustDomainAllocationStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTrustDomainAllocationStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_trust_domain_allocation("code-825", "name-245");
        item.status = FallbackTrustDomainAllocationStatus::Stopped;
        assert!(
            item.move_to(FallbackTrustDomainAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
