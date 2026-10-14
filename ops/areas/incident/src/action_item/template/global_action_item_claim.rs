use super::*;

use acme_kit::text;

/// Where a global action item claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalActionItemClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalActionItemClaimStatus {
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

impl std::fmt::Display for GlobalActionItemClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global action item claim as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalActionItemClaim {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub description: String,
    pub checksum: String,
    pub notes: String,
    pub namespace: String,
    pub floor_value: f64,
    pub created_at: i64,
    pub expires_at: i64,
    pub budget: f64,
    pub status: GlobalActionItemClaimStatus,
}

impl GlobalActionItemClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalActionItemClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalActionItemClaimStatus) -> bool {
        match self.status {
            GlobalActionItemClaimStatus::Idle => {
                match next {
                    GlobalActionItemClaimStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalActionItemClaimStatus::Warming => {
                match next {
                    GlobalActionItemClaimStatus::Ready | GlobalActionItemClaimStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalActionItemClaimStatus::Ready => {
                return next == GlobalActionItemClaimStatus::Draining;
            }
            GlobalActionItemClaimStatus::Draining => {
                return next == GlobalActionItemClaimStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalActionItemClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global action item claim {} cannot move from {} to {}",
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

    pub fn projected_load(&self) -> f64 {
        ((self.budget + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalActionItemClaim {
        GlobalActionItemClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            description: self.description.clone(),
            checksum: self.checksum.clone(),
            notes: self.notes.clone(),
            namespace: self.namespace.clone(),
            floor_value: self.floor_value,
            created_at: self.created_at,
            expires_at: self.expires_at,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global action item claim in status idle.
pub fn new_global_action_item_claim(code: &str, title: &str) -> GlobalActionItemClaim {
    GlobalActionItemClaim {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        description: "".to_string(),
        checksum: "".to_string(),
        notes: "".to_string(),
        namespace: "".to_string(),
        floor_value: 0.0,
        created_at: 0,
        expires_at: 0,
        budget: 0.0,
        status: GlobalActionItemClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_action_item_claim("code-664", "title-783");
        assert_eq!(item.status, GlobalActionItemClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_global_action_item_claim("code-664", "title-783");
        if item.move_to(GlobalActionItemClaimStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalActionItemClaimStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_action_item_claim("code-664", "title-783");
        item.status = GlobalActionItemClaimStatus::Stopped;
        assert!(
            item.move_to(GlobalActionItemClaimStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
