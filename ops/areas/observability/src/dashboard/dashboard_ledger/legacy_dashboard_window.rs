use super::*;

use acme_kit::text;

/// Where a legacy dashboard window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDashboardWindowStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyDashboardWindowStatus {
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

impl std::fmt::Display for LegacyDashboardWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy dashboard window as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardWindow {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub description: String,
    pub key_id: String,
    pub capacity: i64,
    pub port: i64,
    pub country: String,
    pub minimum: f64,
    pub status: LegacyDashboardWindowStatus,
}

impl LegacyDashboardWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDashboardWindowStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDashboardWindowStatus) -> bool {
        match self.status {
            LegacyDashboardWindowStatus::Open => {
                return next == LegacyDashboardWindowStatus::Reopened;
            }
            LegacyDashboardWindowStatus::InReview => {
                match next {
                    LegacyDashboardWindowStatus::Resolved | LegacyDashboardWindowStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDashboardWindowStatus::Resolved => {
                match next {
                    LegacyDashboardWindowStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDashboardWindowStatus::Reopened => {
                match next {
                    LegacyDashboardWindowStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyDashboardWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy dashboard window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy dashboard window", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.minimum + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDashboardWindow {
        LegacyDashboardWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            description: self.description.clone(),
            key_id: self.key_id.clone(),
            capacity: self.capacity,
            port: self.port,
            country: self.country.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy dashboard window in status open.
pub fn new_legacy_dashboard_window(code: &str, label: &str) -> LegacyDashboardWindow {
    LegacyDashboardWindow {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        description: "".to_string(),
        key_id: "".to_string(),
        capacity: 0,
        port: 0,
        country: "".to_string(),
        minimum: 0.0,
        status: LegacyDashboardWindowStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_dashboard_window("code-561", "label-703");
        assert_eq!(item.status, LegacyDashboardWindowStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_legacy_dashboard_window("code-561", "label-703");
        if item.move_to(LegacyDashboardWindowStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDashboardWindowStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_dashboard_window("code-561", "label-703");
        item.status = LegacyDashboardWindowStatus::Archived;
        assert!(
            item.move_to(LegacyDashboardWindowStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
