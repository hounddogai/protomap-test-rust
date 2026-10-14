use super::*;

use acme_kit::text;

/// Where a default tenant assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultTenantAssignmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultTenantAssignmentStatus {
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

impl std::fmt::Display for DefaultTenantAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default tenant assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantAssignment {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub discount: f64,
    pub phone: String,
    pub ceiling: f64,
    pub owner_id: String,
    pub duration_minutes: i64,
    pub amount: f64,
    pub maximum: f64,
    pub window_days: i64,
    pub status: DefaultTenantAssignmentStatus,
}

impl DefaultTenantAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultTenantAssignmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultTenantAssignmentStatus) -> bool {
        match self.status {
            DefaultTenantAssignmentStatus::New => {
                return next == DefaultTenantAssignmentStatus::Queued || next == DefaultTenantAssignmentStatus::Failed;
            }
            DefaultTenantAssignmentStatus::Queued => {
                match next {
                    DefaultTenantAssignmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultTenantAssignmentStatus::Running => {
                return next == DefaultTenantAssignmentStatus::Done || next == DefaultTenantAssignmentStatus::Failed;
            }
            DefaultTenantAssignmentStatus::Done => {
                match next {
                    DefaultTenantAssignmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultTenantAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default tenant assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ceiling - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultTenantAssignment {
        DefaultTenantAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            discount: self.discount,
            phone: self.phone.clone(),
            ceiling: self.ceiling,
            owner_id: self.owner_id.clone(),
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            maximum: self.maximum,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default tenant assignment in status new.
pub fn new_default_tenant_assignment(code: &str, title: &str) -> DefaultTenantAssignment {
    DefaultTenantAssignment {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        discount: 0.0,
        phone: "".to_string(),
        ceiling: 0.0,
        owner_id: "".to_string(),
        duration_minutes: 0,
        amount: 0.0,
        maximum: 0.0,
        window_days: 0,
        status: DefaultTenantAssignmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_tenant_assignment("code-982", "title-26");
        assert_eq!(item.status, DefaultTenantAssignmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_default_tenant_assignment("code-982", "title-26");
        if item.move_to(DefaultTenantAssignmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultTenantAssignmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_tenant_assignment("code-982", "title-26");
        item.status = DefaultTenantAssignmentStatus::Failed;
        assert!(
            item.move_to(DefaultTenantAssignmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
