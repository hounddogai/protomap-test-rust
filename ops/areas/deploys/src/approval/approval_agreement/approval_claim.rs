use super::*;

use acme_kit::text;

/// Where a approval claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ApprovalClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ApprovalClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval claim as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalClaim {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub cluster: String,
    pub tags: Vec<String>,
    pub unit: String,
    pub principal: String,
    pub priority: i64,
    pub maximum: f64,
    pub notes: String,
    pub status: ApprovalClaimStatus,
}

impl ApprovalClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalClaimStatus) -> bool {
        match self.status {
            ApprovalClaimStatus::New => {
                return next == ApprovalClaimStatus::Running;
            }
            ApprovalClaimStatus::Queued => {
                match next {
                    ApprovalClaimStatus::Done | ApprovalClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalClaimStatus::Running => {
                return next == ApprovalClaimStatus::Failed;
            }
            ApprovalClaimStatus::Done => {
                return next == ApprovalClaimStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.priority as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalClaim {
        ApprovalClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            owner_id: self.owner_id.clone(),
            cluster: self.cluster.clone(),
            tags: self.tags.clone(),
            unit: self.unit.clone(),
            principal: self.principal.clone(),
            priority: self.priority,
            maximum: self.maximum,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a approval claim in status new.
pub fn new_approval_claim(code: &str, name: &str) -> ApprovalClaim {
    ApprovalClaim {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        owner_id: "".to_string(),
        cluster: "".to_string(),
        tags: Vec::new(),
        unit: "".to_string(),
        principal: "".to_string(),
        priority: 0,
        maximum: 0.0,
        notes: "".to_string(),
        status: ApprovalClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_approval_claim("code-28", "name-175");
        assert_eq!(item.status, ApprovalClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_approval_claim("code-28", "name-175");
        if item.move_to(ApprovalClaimStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalClaimStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_approval_claim("code-28", "name-175");
        item.status = ApprovalClaimStatus::Failed;
        assert!(item.move_to(ApprovalClaimStatus::New).is_err(), "expected move to fail");
    }
}
