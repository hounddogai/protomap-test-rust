use super::*;

use acme_kit::text;

/// Where a approval allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ApprovalAllocationStatus {
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

impl std::fmt::Display for ApprovalAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocation {
    pub code: String,
    pub title: String,
    pub host: String,
    pub owner_id: String,
    pub tags: Vec<String>,
    pub archived: bool,
    pub tenant_id: String,
    pub reviewer: String,
    pub rate: f64,
    pub status: ApprovalAllocationStatus,
}

impl ApprovalAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalAllocationStatus) -> bool {
        match self.status {
            ApprovalAllocationStatus::Planned => {
                match next {
                    ApprovalAllocationStatus::InProgress | ApprovalAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalAllocationStatus::Scheduled => {
                return next == ApprovalAllocationStatus::InProgress || next == ApprovalAllocationStatus::Cancelled;
            }
            ApprovalAllocationStatus::InProgress => {
                return next == ApprovalAllocationStatus::Completed || next == ApprovalAllocationStatus::Cancelled;
            }
            ApprovalAllocationStatus::Completed => {
                match next {
                    ApprovalAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ApprovalAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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
        self.rate
    }

    pub fn copy(&self) -> ApprovalAllocation {
        ApprovalAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            owner_id: self.owner_id.clone(),
            tags: self.tags.clone(),
            archived: self.archived,
            tenant_id: self.tenant_id.clone(),
            reviewer: self.reviewer.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a approval allocation in status planned.
pub fn new_approval_allocation(code: &str, title: &str) -> ApprovalAllocation {
    ApprovalAllocation {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        owner_id: "".to_string(),
        tags: Vec::new(),
        archived: false,
        tenant_id: "".to_string(),
        reviewer: "".to_string(),
        rate: 0.0,
        status: ApprovalAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_approval_allocation("code-906", "title-56");
        assert_eq!(item.status, ApprovalAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_approval_allocation("code-906", "title-56");
        if item.move_to(ApprovalAllocationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalAllocationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_approval_allocation("code-906", "title-56");
        item.status = ApprovalAllocationStatus::Cancelled;
        assert!(
            item.move_to(ApprovalAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
