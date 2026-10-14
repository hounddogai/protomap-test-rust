use super::*;

use acme_kit::text;

/// Where a primary retention summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRetentionSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryRetentionSummaryStatus {
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

impl std::fmt::Display for PrimaryRetentionSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary retention summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRetentionSummary {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub timezone: String,
    pub score: f64,
    pub segment: String,
    pub grace_days: i64,
    pub summary: String,
    pub scope: String,
    pub window_days: i64,
    pub status: PrimaryRetentionSummaryStatus,
}

impl PrimaryRetentionSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRetentionSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRetentionSummaryStatus) -> bool {
        match self.status {
            PrimaryRetentionSummaryStatus::Draft => {
                return next == PrimaryRetentionSummaryStatus::Pending;
            }
            PrimaryRetentionSummaryStatus::Pending => {
                match next {
                    PrimaryRetentionSummaryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRetentionSummaryStatus::Active => {
                return next == PrimaryRetentionSummaryStatus::Suspended
                    || next == PrimaryRetentionSummaryStatus::Closed;
            }
            PrimaryRetentionSummaryStatus::Suspended => {
                match next {
                    PrimaryRetentionSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryRetentionSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary retention summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.window_days as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRetentionSummary {
        PrimaryRetentionSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            timezone: self.timezone.clone(),
            score: self.score,
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            summary: self.summary.clone(),
            scope: self.scope.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary retention summary in status draft.
pub fn new_primary_retention_summary(code: &str, label: &str) -> PrimaryRetentionSummary {
    PrimaryRetentionSummary {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        timezone: "".to_string(),
        score: 0.0,
        segment: "".to_string(),
        grace_days: 0,
        summary: "".to_string(),
        scope: "".to_string(),
        window_days: 0,
        status: PrimaryRetentionSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_retention_summary("code-666", "label-459");
        assert_eq!(item.status, PrimaryRetentionSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_primary_retention_summary("code-666", "label-459");
        if item.move_to(PrimaryRetentionSummaryStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRetentionSummaryStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_retention_summary("code-666", "label-459");
        item.status = PrimaryRetentionSummaryStatus::Closed;
        assert!(
            item.move_to(PrimaryRetentionSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
