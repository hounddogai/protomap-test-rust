use super::*;

use acme_kit::text;

/// Where a dashboard agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DashboardAgreementStatus {
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

impl std::fmt::Display for DashboardAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard agreement as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardAgreement {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub ends_at: i64,
    pub contact_email: String,
    pub pinned: bool,
    pub cipher: String,
    pub host: String,
    pub window_days: i64,
    pub status: DashboardAgreementStatus,
}

impl DashboardAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardAgreementStatus) -> bool {
        match self.status {
            DashboardAgreementStatus::Planned => {
                match next {
                    DashboardAgreementStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardAgreementStatus::Scheduled => {
                return next == DashboardAgreementStatus::Cancelled;
            }
            DashboardAgreementStatus::InProgress => {
                return next == DashboardAgreementStatus::Completed;
            }
            DashboardAgreementStatus::Completed => {
                match next {
                    DashboardAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DashboardAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard agreement {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardAgreement {
        DashboardAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            ends_at: self.ends_at,
            contact_email: self.contact_email.clone(),
            pinned: self.pinned,
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dashboard agreement in status planned.
pub fn new_dashboard_agreement(code: &str, label: &str) -> DashboardAgreement {
    DashboardAgreement {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        ends_at: 0,
        contact_email: "".to_string(),
        pinned: false,
        cipher: "".to_string(),
        host: "".to_string(),
        window_days: 0,
        status: DashboardAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_dashboard_agreement("code-676", "label-225");
        assert_eq!(item.status, DashboardAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_dashboard_agreement("code-676", "label-225");
        if item.move_to(DashboardAgreementStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardAgreementStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_dashboard_agreement("code-676", "label-225");
        item.status = DashboardAgreementStatus::Cancelled;
        assert!(
            item.move_to(DashboardAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
