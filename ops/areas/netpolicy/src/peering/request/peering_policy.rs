use super::*;

use acme_kit::text;

/// Where a peering policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringPolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PeeringPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PeeringPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPolicy {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub cluster: String,
    pub status: PeeringPolicyStatus,
}

impl PeeringPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringPolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringPolicyStatus) -> bool {
        match self.status {
            PeeringPolicyStatus::Planned => {
                match next {
                    PeeringPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringPolicyStatus::Scheduled => {
                return next == PeeringPolicyStatus::Cancelled;
            }
            PeeringPolicyStatus::InProgress => {
                match next {
                    PeeringPolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringPolicyStatus::Completed => {
                match next {
                    PeeringPolicyStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PeeringPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering policy", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringPolicy {
        PeeringPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a peering policy in status planned.
pub fn new_peering_policy(code: &str, name: &str) -> PeeringPolicy {
    PeeringPolicy {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        ttl_seconds: 0,
        ceiling: 0.0,
        cluster: "".to_string(),
        status: PeeringPolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_peering_policy("code-105", "name-817");
        assert_eq!(item.status, PeeringPolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_peering_policy("code-105", "name-817");
        if item.move_to(PeeringPolicyStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringPolicyStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_peering_policy("code-105", "name-817");
        item.status = PeeringPolicyStatus::Cancelled;
        assert!(
            item.move_to(PeeringPolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
