use super::*;

use acme_kit::text;

/// Where a tenant assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TenantAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAssignment {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub title: String,
    pub sequence: i64,
    pub fingerprint: String,
    pub scope: String,
    pub status: TenantAssignmentStatus,
}

impl TenantAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantAssignmentStatus) -> bool {
        match self.status {
            TenantAssignmentStatus::Open => {
                match next {
                    TenantAssignmentStatus::Reopened | TenantAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAssignmentStatus::InReview => {
                match next {
                    TenantAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAssignmentStatus::Resolved => {
                match next {
                    TenantAssignmentStatus::Reopened | TenantAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAssignmentStatus::Reopened => {
                match next {
                    TenantAssignmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: TenantAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> TenantAssignment {
        TenantAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tenant assignment in status open.
pub fn new_tenant_assignment(code: &str, label: &str) -> TenantAssignment {
    TenantAssignment {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        title: "".to_string(),
        sequence: 0,
        fingerprint: "".to_string(),
        scope: "".to_string(),
        status: TenantAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_assignment("code-119", "label-748");
        assert_eq!(item.status, TenantAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tenant_assignment("code-119", "label-748");
        if item.move_to(TenantAssignmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantAssignmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_assignment("code-119", "label-748");
        item.status = TenantAssignmentStatus::Archived;
        assert!(
            item.move_to(TenantAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
