use super::*;

use acme_kit::text;

/// Where a global postmortem claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPostmortemClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalPostmortemClaimStatus {
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

impl std::fmt::Display for GlobalPostmortemClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global postmortem claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPostmortemClaim {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub limit: i64,
    pub burst: i64,
    pub sequence: i64,
    pub created_at: i64,
    pub audience: String,
    pub ends_at: i64,
    pub visible: bool,
    pub updated_at: i64,
    pub status: GlobalPostmortemClaimStatus,
}

impl GlobalPostmortemClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPostmortemClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPostmortemClaimStatus) -> bool {
        match self.status {
            GlobalPostmortemClaimStatus::Planned => {
                return next == GlobalPostmortemClaimStatus::Scheduled
                    || next == GlobalPostmortemClaimStatus::InProgress;
            }
            GlobalPostmortemClaimStatus::Scheduled => {
                return next == GlobalPostmortemClaimStatus::Cancelled;
            }
            GlobalPostmortemClaimStatus::InProgress => {
                match next {
                    GlobalPostmortemClaimStatus::Completed | GlobalPostmortemClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPostmortemClaimStatus::Completed => {
                match next {
                    GlobalPostmortemClaimStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalPostmortemClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global postmortem claim {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.updated_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPostmortemClaim {
        GlobalPostmortemClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            limit: self.limit,
            burst: self.burst,
            sequence: self.sequence,
            created_at: self.created_at,
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            visible: self.visible,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a global postmortem claim in status planned.
pub fn new_global_postmortem_claim(code: &str, title: &str) -> GlobalPostmortemClaim {
    GlobalPostmortemClaim {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        limit: 0,
        burst: 0,
        sequence: 0,
        created_at: 0,
        audience: "".to_string(),
        ends_at: 0,
        visible: false,
        updated_at: 0,
        status: GlobalPostmortemClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_postmortem_claim("code-293", "title-476");
        assert_eq!(item.status, GlobalPostmortemClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_global_postmortem_claim("code-293", "title-476");
        if item.move_to(GlobalPostmortemClaimStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPostmortemClaimStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_postmortem_claim("code-293", "title-476");
        item.status = GlobalPostmortemClaimStatus::Cancelled;
        assert!(
            item.move_to(GlobalPostmortemClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
