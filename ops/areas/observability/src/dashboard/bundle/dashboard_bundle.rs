use super::*;

use acme_kit::text;

/// Where a dashboard bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardBundleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DashboardBundleStatus {
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

impl std::fmt::Display for DashboardBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBundle {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub source: String,
    pub limit: i64,
    pub notes: String,
    pub title: String,
    pub capacity: i64,
    pub port: i64,
    pub score: f64,
    pub rate: f64,
    pub status: DashboardBundleStatus,
}

impl DashboardBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardBundleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardBundleStatus) -> bool {
        match self.status {
            DashboardBundleStatus::Open => {
                match next {
                    DashboardBundleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardBundleStatus::InReview => {
                match next {
                    DashboardBundleStatus::Resolved | DashboardBundleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardBundleStatus::Resolved => {
                return next == DashboardBundleStatus::Reopened || next == DashboardBundleStatus::Archived;
            }
            DashboardBundleStatus::Reopened => {
                match next {
                    DashboardBundleStatus::Archived => {
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

    pub fn move_to(&mut self, next: DashboardBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dashboard bundle", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.limit as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardBundle {
        DashboardBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            source: self.source.clone(),
            limit: self.limit,
            notes: self.notes.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            port: self.port,
            score: self.score,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a dashboard bundle in status open.
pub fn new_dashboard_bundle(code: &str, label: &str) -> DashboardBundle {
    DashboardBundle {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        source: "".to_string(),
        limit: 0,
        notes: "".to_string(),
        title: "".to_string(),
        capacity: 0,
        port: 0,
        score: 0.0,
        rate: 0.0,
        status: DashboardBundleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dashboard_bundle("code-832", "label-705");
        assert_eq!(item.status, DashboardBundleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_dashboard_bundle("code-832", "label-705");
        if item.move_to(DashboardBundleStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardBundleStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dashboard_bundle("code-832", "label-705");
        item.status = DashboardBundleStatus::Archived;
        assert!(
            item.move_to(DashboardBundleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
