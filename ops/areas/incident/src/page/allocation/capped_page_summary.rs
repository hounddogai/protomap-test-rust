use super::*;

use acme_kit::text;

/// Where a capped page summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPageSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedPageSummaryStatus {
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

impl std::fmt::Display for CappedPageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped page summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageSummary {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub summary: String,
    pub window_days: i64,
    pub scope: String,
    pub tenant_id: String,
    pub segment: String,
    pub rate: f64,
    pub created_at: i64,
    pub contact_email: String,
    pub status: CappedPageSummaryStatus,
}

impl CappedPageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPageSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPageSummaryStatus) -> bool {
        match self.status {
            CappedPageSummaryStatus::Planned => {
                return next == CappedPageSummaryStatus::Scheduled || next == CappedPageSummaryStatus::Cancelled;
            }
            CappedPageSummaryStatus::Scheduled => {
                return next == CappedPageSummaryStatus::InProgress;
            }
            CappedPageSummaryStatus::InProgress => {
                match next {
                    CappedPageSummaryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPageSummaryStatus::Completed => {
                match next {
                    CappedPageSummaryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CappedPageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped page summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.window_days as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPageSummary {
        CappedPageSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            summary: self.summary.clone(),
            window_days: self.window_days,
            scope: self.scope.clone(),
            tenant_id: self.tenant_id.clone(),
            segment: self.segment.clone(),
            rate: self.rate,
            created_at: self.created_at,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a capped page summary in status planned.
pub fn new_capped_page_summary(code: &str, label: &str) -> CappedPageSummary {
    CappedPageSummary {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        summary: "".to_string(),
        window_days: 0,
        scope: "".to_string(),
        tenant_id: "".to_string(),
        segment: "".to_string(),
        rate: 0.0,
        created_at: 0,
        contact_email: "".to_string(),
        status: CappedPageSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_page_summary("code-730", "label-640");
        assert_eq!(item.status, CappedPageSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_capped_page_summary("code-730", "label-640");
        if item.move_to(CappedPageSummaryStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPageSummaryStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_page_summary("code-730", "label-640");
        item.status = CappedPageSummaryStatus::Cancelled;
        assert!(
            item.move_to(CappedPageSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
