use super::*;

use acme_kit::text;

/// Where a pending dashboard baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingDashboardBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingDashboardBaselineStatus {
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

impl std::fmt::Display for PendingDashboardBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending dashboard baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBaseline {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub threshold: f64,
    pub priority: i64,
    pub issuer: String,
    pub status: PendingDashboardBaselineStatus,
}

impl PendingDashboardBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingDashboardBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingDashboardBaselineStatus) -> bool {
        match self.status {
            PendingDashboardBaselineStatus::Open => {
                match next {
                    PendingDashboardBaselineStatus::Resolved | PendingDashboardBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDashboardBaselineStatus::InReview => {
                return next == PendingDashboardBaselineStatus::Archived;
            }
            PendingDashboardBaselineStatus::Resolved => {
                match next {
                    PendingDashboardBaselineStatus::Reopened | PendingDashboardBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingDashboardBaselineStatus::Reopened => {
                match next {
                    PendingDashboardBaselineStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingDashboardBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending dashboard baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.updated_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingDashboardBaseline {
        PendingDashboardBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            threshold: self.threshold,
            priority: self.priority,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending dashboard baseline in status open.
pub fn new_pending_dashboard_baseline(code: &str, title: &str) -> PendingDashboardBaseline {
    PendingDashboardBaseline {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        threshold: 0.0,
        priority: 0,
        issuer: "".to_string(),
        status: PendingDashboardBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_dashboard_baseline("code-343", "title-518");
        assert_eq!(item.status, PendingDashboardBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_pending_dashboard_baseline("code-343", "title-518");
        if item.move_to(PendingDashboardBaselineStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingDashboardBaselineStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_dashboard_baseline("code-343", "title-518");
        item.status = PendingDashboardBaselineStatus::Archived;
        assert!(
            item.move_to(PendingDashboardBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
