use super::*;

use acme_kit::text;

/// Where a adaptive credential tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveCredentialTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveCredentialTierStatus {
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

impl std::fmt::Display for AdaptiveCredentialTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive credential tier as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialTier {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub margin: f64,
    pub owner_id: String,
    pub title: String,
    pub expires_at: i64,
    pub rotation_days: i64,
    pub status: AdaptiveCredentialTierStatus,
}

impl AdaptiveCredentialTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveCredentialTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveCredentialTierStatus) -> bool {
        match self.status {
            AdaptiveCredentialTierStatus::Idle => {
                return next == AdaptiveCredentialTierStatus::Warming || next == AdaptiveCredentialTierStatus::Stopped;
            }
            AdaptiveCredentialTierStatus::Warming => {
                return next == AdaptiveCredentialTierStatus::Ready || next == AdaptiveCredentialTierStatus::Draining;
            }
            AdaptiveCredentialTierStatus::Ready => {
                return next == AdaptiveCredentialTierStatus::Stopped;
            }
            AdaptiveCredentialTierStatus::Draining => {
                return next == AdaptiveCredentialTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveCredentialTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive credential tier {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.expires_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveCredentialTier {
        AdaptiveCredentialTier {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            margin: self.margin,
            owner_id: self.owner_id.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive credential tier in status idle.
pub fn new_adaptive_credential_tier(code: &str, name: &str) -> AdaptiveCredentialTier {
    AdaptiveCredentialTier {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        margin: 0.0,
        owner_id: "".to_string(),
        title: "".to_string(),
        expires_at: 0,
        rotation_days: 0,
        status: AdaptiveCredentialTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_credential_tier("code-183", "name-160");
        assert_eq!(item.status, AdaptiveCredentialTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_adaptive_credential_tier("code-183", "name-160");
        if item.move_to(AdaptiveCredentialTierStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveCredentialTierStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_credential_tier("code-183", "name-160");
        item.status = AdaptiveCredentialTierStatus::Stopped;
        assert!(
            item.move_to(AdaptiveCredentialTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
