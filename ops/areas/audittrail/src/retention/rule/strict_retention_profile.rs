use super::*;

use acme_kit::text;

/// Where a strict retention profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRetentionProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictRetentionProfileStatus {
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

impl std::fmt::Display for StrictRetentionProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict retention profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionProfile {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub amount: f64,
    pub revision: i64,
    pub updated_at: i64,
    pub contact_email: String,
    pub version: i64,
    pub cipher: String,
    pub audience: String,
    pub status: StrictRetentionProfileStatus,
}

impl StrictRetentionProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRetentionProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRetentionProfileStatus) -> bool {
        match self.status {
            StrictRetentionProfileStatus::Idle => {
                return next == StrictRetentionProfileStatus::Warming || next == StrictRetentionProfileStatus::Ready;
            }
            StrictRetentionProfileStatus::Warming => {
                return next == StrictRetentionProfileStatus::Draining;
            }
            StrictRetentionProfileStatus::Ready => {
                return next == StrictRetentionProfileStatus::Draining;
            }
            StrictRetentionProfileStatus::Draining => {
                match next {
                    StrictRetentionProfileStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictRetentionProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict retention profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.version as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRetentionProfile {
        StrictRetentionProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            amount: self.amount,
            revision: self.revision,
            updated_at: self.updated_at,
            contact_email: self.contact_email.clone(),
            version: self.version,
            cipher: self.cipher.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a strict retention profile in status idle.
pub fn new_strict_retention_profile(code: &str, title: &str) -> StrictRetentionProfile {
    StrictRetentionProfile {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        amount: 0.0,
        revision: 0,
        updated_at: 0,
        contact_email: "".to_string(),
        version: 0,
        cipher: "".to_string(),
        audience: "".to_string(),
        status: StrictRetentionProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_retention_profile("code-973", "title-839");
        assert_eq!(item.status, StrictRetentionProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_retention_profile("code-973", "title-839");
        if item.move_to(StrictRetentionProfileStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRetentionProfileStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_retention_profile("code-973", "title-839");
        item.status = StrictRetentionProfileStatus::Stopped;
        assert!(
            item.move_to(StrictRetentionProfileStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
