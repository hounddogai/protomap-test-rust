use super::*;

use acme_kit::text;

/// Where a rolling limit claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingLimitClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingLimitClaimStatus {
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

impl std::fmt::Display for RollingLimitClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling limit claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitClaim {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub active: bool,
    pub checksum: String,
    pub tags: Vec<String>,
    pub contact_email: String,
    pub status: RollingLimitClaimStatus,
}

impl RollingLimitClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingLimitClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingLimitClaimStatus) -> bool {
        match self.status {
            RollingLimitClaimStatus::Idle => {
                return next == RollingLimitClaimStatus::Warming || next == RollingLimitClaimStatus::Ready;
            }
            RollingLimitClaimStatus::Warming => {
                match next {
                    RollingLimitClaimStatus::Draining | RollingLimitClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingLimitClaimStatus::Ready => {
                return next == RollingLimitClaimStatus::Draining;
            }
            RollingLimitClaimStatus::Draining => {
                return next == RollingLimitClaimStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingLimitClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling limit claim {} cannot move from {} to {}",
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
        format!("{} {}", "rolling limit claim", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> RollingLimitClaim {
        RollingLimitClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            active: self.active,
            checksum: self.checksum.clone(),
            tags: self.tags.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling limit claim in status idle.
pub fn new_rolling_limit_claim(code: &str, name: &str) -> RollingLimitClaim {
    RollingLimitClaim {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        active: false,
        checksum: "".to_string(),
        tags: Vec::new(),
        contact_email: "".to_string(),
        status: RollingLimitClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_limit_claim("code-848", "name-240");
        assert_eq!(item.status, RollingLimitClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_rolling_limit_claim("code-848", "name-240");
        if item.move_to(RollingLimitClaimStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingLimitClaimStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_limit_claim("code-848", "name-240");
        item.status = RollingLimitClaimStatus::Stopped;
        assert!(
            item.move_to(RollingLimitClaimStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
