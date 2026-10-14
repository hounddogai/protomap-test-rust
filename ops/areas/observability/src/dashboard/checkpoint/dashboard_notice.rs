use super::*;

use acme_kit::text;

/// Where a dashboard notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DashboardNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DashboardNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardNotice {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub margin: f64,
    pub country: String,
    pub weight: f64,
    pub status: DashboardNoticeStatus,
}

impl DashboardNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardNoticeStatus) -> bool {
        match self.status {
            DashboardNoticeStatus::Proposed => {
                return next == DashboardNoticeStatus::Withdrawn;
            }
            DashboardNoticeStatus::Approved => {
                match next {
                    DashboardNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardNoticeStatus::Rejected => {
                return next == DashboardNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        (self.weight * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardNotice {
        DashboardNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            margin: self.margin,
            country: self.country.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a dashboard notice in status proposed.
pub fn new_dashboard_notice(code: &str, label: &str) -> DashboardNotice {
    DashboardNotice {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        margin: 0.0,
        country: "".to_string(),
        weight: 0.0,
        status: DashboardNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_dashboard_notice("code-496", "label-692");
        assert_eq!(item.status, DashboardNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_dashboard_notice("code-496", "label-692");
        if item.move_to(DashboardNoticeStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardNoticeStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_dashboard_notice("code-496", "label-692");
        item.status = DashboardNoticeStatus::Withdrawn;
        assert!(
            item.move_to(DashboardNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
