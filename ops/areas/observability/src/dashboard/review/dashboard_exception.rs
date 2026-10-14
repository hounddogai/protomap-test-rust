use super::*;

use acme_kit::text;

/// Where a dashboard exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DashboardExceptionStatus {
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

impl std::fmt::Display for DashboardExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dashboard exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardException {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub archived: bool,
    pub starts_at: i64,
    pub rotation_days: i64,
    pub status: DashboardExceptionStatus,
}

impl DashboardException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardExceptionStatus) -> bool {
        match self.status {
            DashboardExceptionStatus::Open => {
                match next {
                    DashboardExceptionStatus::Resolved | DashboardExceptionStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardExceptionStatus::InReview => {
                return next == DashboardExceptionStatus::Resolved || next == DashboardExceptionStatus::Reopened;
            }
            DashboardExceptionStatus::Resolved => {
                return next == DashboardExceptionStatus::Archived;
            }
            DashboardExceptionStatus::Reopened => {
                match next {
                    DashboardExceptionStatus::Archived => {
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

    pub fn move_to(&mut self, next: DashboardExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.rotation_days as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardException {
        DashboardException {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            archived: self.archived,
            starts_at: self.starts_at,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dashboard exception in status open.
pub fn new_dashboard_exception(code: &str, name: &str) -> DashboardException {
    DashboardException {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        archived: false,
        starts_at: 0,
        rotation_days: 0,
        status: DashboardExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dashboard_exception("code-715", "name-14");
        assert_eq!(item.status, DashboardExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_dashboard_exception("code-715", "name-14");
        if item.move_to(DashboardExceptionStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardExceptionStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dashboard_exception("code-715", "name-14");
        item.status = DashboardExceptionStatus::Archived;
        assert!(
            item.move_to(DashboardExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
