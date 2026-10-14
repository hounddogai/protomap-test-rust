use super::*;

use acme_kit::text;

/// Where a alert summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertSummaryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AlertSummaryStatus {
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

impl std::fmt::Display for AlertSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert summary as the alert team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertSummary {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub checksum: String,
    pub duration_minutes: i64,
    pub notes: String,
    pub approved: bool,
    pub threshold: f64,
    pub contact_email: String,
    pub budget: f64,
    pub sequence: i64,
    pub status: AlertSummaryStatus,
}

impl AlertSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertSummaryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertSummaryStatus) -> bool {
        match self.status {
            AlertSummaryStatus::Draft => {
                return next == AlertSummaryStatus::Suspended;
            }
            AlertSummaryStatus::Pending => {
                return next == AlertSummaryStatus::Suspended;
            }
            AlertSummaryStatus::Active => {
                return next == AlertSummaryStatus::Closed;
            }
            AlertSummaryStatus::Suspended => {
                match next {
                    AlertSummaryStatus::Closed => {
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

    pub fn move_to(&mut self, next: AlertSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        (self.threshold * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertSummary {
        AlertSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            checksum: self.checksum.clone(),
            duration_minutes: self.duration_minutes,
            notes: self.notes.clone(),
            approved: self.approved,
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            budget: self.budget,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a alert summary in status draft.
pub fn new_alert_summary(code: &str, title: &str) -> AlertSummary {
    AlertSummary {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        checksum: "".to_string(),
        duration_minutes: 0,
        notes: "".to_string(),
        approved: false,
        threshold: 0.0,
        contact_email: "".to_string(),
        budget: 0.0,
        sequence: 0,
        status: AlertSummaryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_alert_summary("code-636", "title-77");
        assert_eq!(item.status, AlertSummaryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_alert_summary("code-636", "title-77");
        if item.move_to(AlertSummaryStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertSummaryStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_alert_summary("code-636", "title-77");
        item.status = AlertSummaryStatus::Closed;
        assert!(
            item.move_to(AlertSummaryStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
