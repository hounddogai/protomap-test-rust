use super::*;

use acme_kit::text;

/// Where a approval audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ApprovalAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ApprovalAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A approval audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAudit {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub scope: String,
    pub description: String,
    pub namespace: String,
    pub score: f64,
    pub status: ApprovalAuditStatus,
}

impl ApprovalAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalAuditStatus) -> bool {
        match self.status {
            ApprovalAuditStatus::Draft => {
                match next {
                    ApprovalAuditStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalAuditStatus::Pending => {
                return next == ApprovalAuditStatus::Active || next == ApprovalAuditStatus::Closed;
            }
            ApprovalAuditStatus::Active => {
                return next == ApprovalAuditStatus::Suspended || next == ApprovalAuditStatus::Closed;
            }
            ApprovalAuditStatus::Suspended => {
                match next {
                    ApprovalAuditStatus::Closed => {
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

    pub fn move_to(&mut self, next: ApprovalAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ApprovalAudit {
        ApprovalAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            scope: self.scope.clone(),
            description: self.description.clone(),
            namespace: self.namespace.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a approval audit in status draft.
pub fn new_approval_audit(code: &str, title: &str) -> ApprovalAudit {
    ApprovalAudit {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        scope: "".to_string(),
        description: "".to_string(),
        namespace: "".to_string(),
        score: 0.0,
        status: ApprovalAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_approval_audit("code-739", "title-107");
        assert_eq!(item.status, ApprovalAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_approval_audit("code-739", "title-107");
        if item.move_to(ApprovalAuditStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalAuditStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_approval_audit("code-739", "title-107");
        item.status = ApprovalAuditStatus::Closed;
        assert!(
            item.move_to(ApprovalAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
