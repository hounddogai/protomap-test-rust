use super::*;

use acme_kit::text;

/// Where a pending postmortem policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPostmortemPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingPostmortemPolicyStatus {
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

impl std::fmt::Display for PendingPostmortemPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending postmortem policy as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemPolicy {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub label: String,
    pub cost: f64,
    pub principal: String,
    pub description: String,
    pub status: PendingPostmortemPolicyStatus,
}

impl PendingPostmortemPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPostmortemPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPostmortemPolicyStatus) -> bool {
        match self.status {
            PendingPostmortemPolicyStatus::Idle => {
                return next == PendingPostmortemPolicyStatus::Draining
                    || next == PendingPostmortemPolicyStatus::Stopped;
            }
            PendingPostmortemPolicyStatus::Warming => {
                return next == PendingPostmortemPolicyStatus::Ready;
            }
            PendingPostmortemPolicyStatus::Ready => {
                match next {
                    PendingPostmortemPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPostmortemPolicyStatus::Draining => {
                match next {
                    PendingPostmortemPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingPostmortemPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending postmortem policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> PendingPostmortemPolicy {
        PendingPostmortemPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            label: self.label.clone(),
            cost: self.cost,
            principal: self.principal.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending postmortem policy in status idle.
pub fn new_pending_postmortem_policy(code: &str, title: &str) -> PendingPostmortemPolicy {
    PendingPostmortemPolicy {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        label: "".to_string(),
        cost: 0.0,
        principal: "".to_string(),
        description: "".to_string(),
        status: PendingPostmortemPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_postmortem_policy("code-485", "title-67");
        assert_eq!(item.status, PendingPostmortemPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_pending_postmortem_policy("code-485", "title-67");
        if item.move_to(PendingPostmortemPolicyStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPostmortemPolicyStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_postmortem_policy("code-485", "title-67");
        item.status = PendingPostmortemPolicyStatus::Stopped;
        assert!(
            item.move_to(PendingPostmortemPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
