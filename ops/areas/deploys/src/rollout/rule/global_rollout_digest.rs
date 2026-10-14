use super::*;

use acme_kit::text;

/// Where a global rollout digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRolloutDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalRolloutDigestStatus {
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

impl std::fmt::Display for GlobalRolloutDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global rollout digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRolloutDigest {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub revision: i64,
    pub margin: f64,
    pub country: String,
    pub duration_minutes: i64,
    pub status: GlobalRolloutDigestStatus,
}

impl GlobalRolloutDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRolloutDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRolloutDigestStatus) -> bool {
        match self.status {
            GlobalRolloutDigestStatus::Idle => {
                match next {
                    GlobalRolloutDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRolloutDigestStatus::Warming => {
                return next == GlobalRolloutDigestStatus::Ready;
            }
            GlobalRolloutDigestStatus::Ready => {
                match next {
                    GlobalRolloutDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRolloutDigestStatus::Draining => {
                return next == GlobalRolloutDigestStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRolloutDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global rollout digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRolloutDigest {
        GlobalRolloutDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            margin: self.margin,
            country: self.country.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a global rollout digest in status idle.
pub fn new_global_rollout_digest(code: &str, name: &str) -> GlobalRolloutDigest {
    GlobalRolloutDigest {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        revision: 0,
        margin: 0.0,
        country: "".to_string(),
        duration_minutes: 0,
        status: GlobalRolloutDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_rollout_digest("code-449", "name-828");
        assert_eq!(item.status, GlobalRolloutDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_global_rollout_digest("code-449", "name-828");
        if item.move_to(GlobalRolloutDigestStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRolloutDigestStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_rollout_digest("code-449", "name-828");
        item.status = GlobalRolloutDigestStatus::Stopped;
        assert!(
            item.move_to(GlobalRolloutDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
