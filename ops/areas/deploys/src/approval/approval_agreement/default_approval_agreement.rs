use super::*;

use acme_kit::text;

/// Where a default approval agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultApprovalAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultApprovalAgreementStatus {
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

impl std::fmt::Display for DefaultApprovalAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default approval agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultApprovalAgreement {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub approved: bool,
    pub window_days: i64,
    pub starts_at: i64,
    pub label: String,
    pub margin: f64,
    pub summary: String,
    pub ratio: f64,
    pub quantity: i64,
    pub status: DefaultApprovalAgreementStatus,
}

impl DefaultApprovalAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultApprovalAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultApprovalAgreementStatus) -> bool {
        match self.status {
            DefaultApprovalAgreementStatus::Planned => {
                return next == DefaultApprovalAgreementStatus::InProgress;
            }
            DefaultApprovalAgreementStatus::Scheduled => {
                return next == DefaultApprovalAgreementStatus::InProgress;
            }
            DefaultApprovalAgreementStatus::InProgress => {
                return next == DefaultApprovalAgreementStatus::Cancelled;
            }
            DefaultApprovalAgreementStatus::Completed => {
                match next {
                    DefaultApprovalAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultApprovalAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default approval agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.quantity as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultApprovalAgreement {
        DefaultApprovalAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            approved: self.approved,
            window_days: self.window_days,
            starts_at: self.starts_at,
            label: self.label.clone(),
            margin: self.margin,
            summary: self.summary.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default approval agreement in status planned.
pub fn new_default_approval_agreement(code: &str, title: &str) -> DefaultApprovalAgreement {
    DefaultApprovalAgreement {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        approved: false,
        window_days: 0,
        starts_at: 0,
        label: "".to_string(),
        margin: 0.0,
        summary: "".to_string(),
        ratio: 0.0,
        quantity: 0,
        status: DefaultApprovalAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_approval_agreement("code-402", "title-446");
        assert_eq!(item.status, DefaultApprovalAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_approval_agreement("code-402", "title-446");
        if item.move_to(DefaultApprovalAgreementStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultApprovalAgreementStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_approval_agreement("code-402", "title-446");
        item.status = DefaultApprovalAgreementStatus::Cancelled;
        assert!(
            item.move_to(DefaultApprovalAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
