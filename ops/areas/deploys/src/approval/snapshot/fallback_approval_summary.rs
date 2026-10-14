use super::*;

use acme_kit::text;

/// Where a fallback approval summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackApprovalSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackApprovalSummaryStatus {
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

impl std::fmt::Display for FallbackApprovalSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback approval summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalSummary {
    pub code: String,
    pub label: String,
    pub country: String,
    pub notes: String,
    pub duration_minutes: i64,
    pub cluster: String,
    pub summary: String,
    pub grace_days: i64,
    pub status: FallbackApprovalSummaryStatus,
}

impl FallbackApprovalSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackApprovalSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackApprovalSummaryStatus) -> bool {
        match self.status {
            FallbackApprovalSummaryStatus::Draft => {
                match next {
                    FallbackApprovalSummaryStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackApprovalSummaryStatus::Pending => {
                return next == FallbackApprovalSummaryStatus::Active;
            }
            FallbackApprovalSummaryStatus::Active => {
                match next {
                    FallbackApprovalSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackApprovalSummaryStatus::Suspended => {
                match next {
                    FallbackApprovalSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackApprovalSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback approval summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback approval summary", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackApprovalSummary {
        FallbackApprovalSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            notes: self.notes.clone(),
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            summary: self.summary.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback approval summary in status draft.
pub fn new_fallback_approval_summary(code: &str, label: &str) -> FallbackApprovalSummary {
    FallbackApprovalSummary {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        notes: "".to_string(),
        duration_minutes: 0,
        cluster: "".to_string(),
        summary: "".to_string(),
        grace_days: 0,
        status: FallbackApprovalSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_approval_summary("code-997", "label-904");
        assert_eq!(item.status, FallbackApprovalSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_fallback_approval_summary("code-997", "label-904");
        if item.move_to(FallbackApprovalSummaryStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackApprovalSummaryStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_approval_summary("code-997", "label-904");
        item.status = FallbackApprovalSummaryStatus::Closed;
        assert!(
            item.move_to(FallbackApprovalSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
