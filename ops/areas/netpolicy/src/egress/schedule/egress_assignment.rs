use super::*;

use acme_kit::text;

/// Where a egress assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EgressAssignmentStatus {
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

impl std::fmt::Display for EgressAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressAssignment {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub contact_email: String,
    pub priority: i64,
    pub cluster: String,
    pub key_id: String,
    pub status: EgressAssignmentStatus,
}

impl EgressAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressAssignmentStatus) -> bool {
        match self.status {
            EgressAssignmentStatus::New => {
                match next {
                    EgressAssignmentStatus::Done | EgressAssignmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressAssignmentStatus::Queued => {
                match next {
                    EgressAssignmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressAssignmentStatus::Running => {
                match next {
                    EgressAssignmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressAssignmentStatus::Done => {
                match next {
                    EgressAssignmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: EgressAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> EgressAssignment {
        EgressAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a egress assignment in status new.
pub fn new_egress_assignment(code: &str, title: &str) -> EgressAssignment {
    EgressAssignment {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        contact_email: "".to_string(),
        priority: 0,
        cluster: "".to_string(),
        key_id: "".to_string(),
        status: EgressAssignmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_egress_assignment("code-127", "title-148");
        assert_eq!(item.status, EgressAssignmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_egress_assignment("code-127", "title-148");
        if item.move_to(EgressAssignmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressAssignmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_egress_assignment("code-127", "title-148");
        item.status = EgressAssignmentStatus::Failed;
        assert!(
            item.move_to(EgressAssignmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
