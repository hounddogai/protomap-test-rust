use super::*;

use acme_kit::text;

/// Where a dashboard summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DashboardSummaryStatus {
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

impl std::fmt::Display for DashboardSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard summary as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSummary {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub ratio: f64,
    pub ends_at: i64,
    pub window_days: i64,
    pub attempts: i64,
    pub contact_email: String,
    pub status: DashboardSummaryStatus,
}

impl DashboardSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardSummaryStatus) -> bool {
        match self.status {
            DashboardSummaryStatus::Open => {
                match next {
                    DashboardSummaryStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardSummaryStatus::InReview => {
                return next == DashboardSummaryStatus::Reopened || next == DashboardSummaryStatus::Archived;
            }
            DashboardSummaryStatus::Resolved => {
                return next == DashboardSummaryStatus::Reopened;
            }
            DashboardSummaryStatus::Reopened => {
                return next == DashboardSummaryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard summary {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.attempts as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardSummary {
        DashboardSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            ratio: self.ratio,
            ends_at: self.ends_at,
            window_days: self.window_days,
            attempts: self.attempts,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a dashboard summary in status open.
pub fn new_dashboard_summary(code: &str, label: &str) -> DashboardSummary {
    DashboardSummary {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        ratio: 0.0,
        ends_at: 0,
        window_days: 0,
        attempts: 0,
        contact_email: "".to_string(),
        status: DashboardSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dashboard_summary("code-609", "label-524");
        assert_eq!(item.status, DashboardSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_dashboard_summary("code-609", "label-524");
        if item.move_to(DashboardSummaryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardSummaryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dashboard_summary("code-609", "label-524");
        item.status = DashboardSummaryStatus::Archived;
        assert!(
            item.move_to(DashboardSummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
