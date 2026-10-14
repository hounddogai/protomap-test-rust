use super::*;

use acme_kit::text;

/// Where a quote summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QuoteSummaryStatus {
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

impl std::fmt::Display for QuoteSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A quote summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSummary {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub fingerprint: String,
    pub burst: i64,
    pub label: String,
    pub weight: f64,
    pub principal: String,
    pub unit: String,
    pub status: QuoteSummaryStatus,
}

impl QuoteSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteSummaryStatus) -> bool {
        match self.status {
            QuoteSummaryStatus::Planned => {
                match next {
                    QuoteSummaryStatus::InProgress | QuoteSummaryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSummaryStatus::Scheduled => {
                match next {
                    QuoteSummaryStatus::InProgress | QuoteSummaryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSummaryStatus::InProgress => {
                match next {
                    QuoteSummaryStatus::Completed | QuoteSummaryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSummaryStatus::Completed => {
                match next {
                    QuoteSummaryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: QuoteSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "quote summary", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.burst as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteSummary {
        QuoteSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            label: self.label.clone(),
            weight: self.weight,
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a quote summary in status planned.
pub fn new_quote_summary(code: &str, name: &str) -> QuoteSummary {
    QuoteSummary {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        fingerprint: "".to_string(),
        burst: 0,
        label: "".to_string(),
        weight: 0.0,
        principal: "".to_string(),
        unit: "".to_string(),
        status: QuoteSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_quote_summary("code-31", "name-689");
        assert_eq!(item.status, QuoteSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_quote_summary("code-31", "name-689");
        if item.move_to(QuoteSummaryStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteSummaryStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_quote_summary("code-31", "name-689");
        item.status = QuoteSummaryStatus::Cancelled;
        assert!(
            item.move_to(QuoteSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
