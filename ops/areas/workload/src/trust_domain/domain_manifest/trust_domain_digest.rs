use super::*;

use acme_kit::text;

/// Where a trust domain digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TrustDomainDigestStatus {
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

impl std::fmt::Display for TrustDomainDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDigest {
    pub code: String,
    pub name: String,
    pub category: String,
    pub floor_value: f64,
    pub updated_at: i64,
    pub batch_size: i64,
    pub ceiling: f64,
    pub status: TrustDomainDigestStatus,
}

impl TrustDomainDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainDigestStatus) -> bool {
        match self.status {
            TrustDomainDigestStatus::Idle => {
                match next {
                    TrustDomainDigestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainDigestStatus::Warming => {
                match next {
                    TrustDomainDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainDigestStatus::Ready => {
                match next {
                    TrustDomainDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainDigestStatus::Draining => {
                match next {
                    TrustDomainDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TrustDomainDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain digest {} cannot move from {} to {}",
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
        format!("{} {}", "trust domain digest", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ceiling - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainDigest {
        TrustDomainDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            floor_value: self.floor_value,
            updated_at: self.updated_at,
            batch_size: self.batch_size,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a trust domain digest in status idle.
pub fn new_trust_domain_digest(code: &str, name: &str) -> TrustDomainDigest {
    TrustDomainDigest {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        floor_value: 0.0,
        updated_at: 0,
        batch_size: 0,
        ceiling: 0.0,
        status: TrustDomainDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_trust_domain_digest("code-508", "name-365");
        assert_eq!(item.status, TrustDomainDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_trust_domain_digest("code-508", "name-365");
        if item.move_to(TrustDomainDigestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainDigestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_trust_domain_digest("code-508", "name-365");
        item.status = TrustDomainDigestStatus::Stopped;
        assert!(
            item.move_to(TrustDomainDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
