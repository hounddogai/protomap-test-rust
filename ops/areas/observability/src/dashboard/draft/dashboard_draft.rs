use super::*;

use acme_kit::text;

/// Where a dashboard draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DashboardDraftStatus {
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

impl std::fmt::Display for DashboardDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDraft {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub capacity: i64,
    pub discount: f64,
    pub status: DashboardDraftStatus,
}

impl DashboardDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardDraftStatus) -> bool {
        match self.status {
            DashboardDraftStatus::Draft => {
                return next == DashboardDraftStatus::Pending || next == DashboardDraftStatus::Active;
            }
            DashboardDraftStatus::Pending => {
                return next == DashboardDraftStatus::Suspended || next == DashboardDraftStatus::Closed;
            }
            DashboardDraftStatus::Active => {
                match next {
                    DashboardDraftStatus::Suspended | DashboardDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardDraftStatus::Suspended => {
                match next {
                    DashboardDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: DashboardDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.capacity as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardDraft {
        DashboardDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            capacity: self.capacity,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dashboard draft in status draft.
pub fn new_dashboard_draft(code: &str, title: &str) -> DashboardDraft {
    DashboardDraft {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        capacity: 0,
        discount: 0.0,
        status: DashboardDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dashboard_draft("code-777", "title-875");
        assert_eq!(item.status, DashboardDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_dashboard_draft("code-777", "title-875");
        if item.move_to(DashboardDraftStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardDraftStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dashboard_draft("code-777", "title-875");
        item.status = DashboardDraftStatus::Closed;
        assert!(
            item.move_to(DashboardDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
