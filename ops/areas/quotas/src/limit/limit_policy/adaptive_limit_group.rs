use super::*;

use acme_kit::text;

/// Where a adaptive limit group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveLimitGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveLimitGroupStatus {
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

impl std::fmt::Display for AdaptiveLimitGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive limit group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveLimitGroup {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub principal: String,
    pub notes: String,
    pub created_at: i64,
    pub cipher: String,
    pub status: AdaptiveLimitGroupStatus,
}

impl AdaptiveLimitGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveLimitGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveLimitGroupStatus) -> bool {
        match self.status {
            AdaptiveLimitGroupStatus::Idle => {
                match next {
                    AdaptiveLimitGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveLimitGroupStatus::Warming => {
                match next {
                    AdaptiveLimitGroupStatus::Ready | AdaptiveLimitGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveLimitGroupStatus::Ready => {
                return next == AdaptiveLimitGroupStatus::Draining;
            }
            AdaptiveLimitGroupStatus::Draining => {
                return next == AdaptiveLimitGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveLimitGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive limit group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> AdaptiveLimitGroup {
        AdaptiveLimitGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            notes: self.notes.clone(),
            created_at: self.created_at,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive limit group in status idle.
pub fn new_adaptive_limit_group(code: &str, title: &str) -> AdaptiveLimitGroup {
    AdaptiveLimitGroup {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        principal: "".to_string(),
        notes: "".to_string(),
        created_at: 0,
        cipher: "".to_string(),
        status: AdaptiveLimitGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_limit_group("code-266", "title-760");
        assert_eq!(item.status, AdaptiveLimitGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_adaptive_limit_group("code-266", "title-760");
        if item.move_to(AdaptiveLimitGroupStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveLimitGroupStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_limit_group("code-266", "title-760");
        item.status = AdaptiveLimitGroupStatus::Stopped;
        assert!(
            item.move_to(AdaptiveLimitGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
