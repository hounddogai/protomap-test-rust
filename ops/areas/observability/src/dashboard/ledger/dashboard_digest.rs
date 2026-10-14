use super::*;

use acme_kit::text;

/// Where a dashboard digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DashboardDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DashboardDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDigest {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub region: String,
    pub timezone: String,
    pub archived: bool,
    pub batch_size: i64,
    pub currency: String,
    pub status: DashboardDigestStatus,
}

impl DashboardDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardDigestStatus) -> bool {
        match self.status {
            DashboardDigestStatus::Proposed => {
                return next == DashboardDigestStatus::Approved || next == DashboardDigestStatus::Withdrawn;
            }
            DashboardDigestStatus::Approved => {
                match next {
                    DashboardDigestStatus::Rejected | DashboardDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardDigestStatus::Rejected => {
                return next == DashboardDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard digest {} cannot move from {} to {}",
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
        self.batch_size as f64
    }

    pub fn copy(&self) -> DashboardDigest {
        DashboardDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            archived: self.archived,
            batch_size: self.batch_size,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a dashboard digest in status proposed.
pub fn new_dashboard_digest(code: &str, label: &str) -> DashboardDigest {
    DashboardDigest {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        region: "".to_string(),
        timezone: "".to_string(),
        archived: false,
        batch_size: 0,
        currency: "".to_string(),
        status: DashboardDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_dashboard_digest("code-458", "label-951");
        assert_eq!(item.status, DashboardDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_dashboard_digest("code-458", "label-951");
        if item.move_to(DashboardDigestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardDigestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_dashboard_digest("code-458", "label-951");
        item.status = DashboardDigestStatus::Withdrawn;
        assert!(
            item.move_to(DashboardDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
