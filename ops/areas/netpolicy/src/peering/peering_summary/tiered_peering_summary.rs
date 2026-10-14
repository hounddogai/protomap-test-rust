use super::*;

use acme_kit::text;

/// Where a tiered peering summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPeeringSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredPeeringSummaryStatus {
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

impl std::fmt::Display for TieredPeeringSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered peering summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringSummary {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub scope: String,
    pub budget: f64,
    pub retries: i64,
    pub status: TieredPeeringSummaryStatus,
}

impl TieredPeeringSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPeeringSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPeeringSummaryStatus) -> bool {
        match self.status {
            TieredPeeringSummaryStatus::Planned => {
                return next == TieredPeeringSummaryStatus::Completed;
            }
            TieredPeeringSummaryStatus::Scheduled => {
                match next {
                    TieredPeeringSummaryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPeeringSummaryStatus::InProgress => {
                return next == TieredPeeringSummaryStatus::Cancelled;
            }
            TieredPeeringSummaryStatus::Completed => {
                match next {
                    TieredPeeringSummaryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredPeeringSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered peering summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.budget + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPeeringSummary {
        TieredPeeringSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            scope: self.scope.clone(),
            budget: self.budget,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered peering summary in status planned.
pub fn new_tiered_peering_summary(code: &str, name: &str) -> TieredPeeringSummary {
    TieredPeeringSummary {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        scope: "".to_string(),
        budget: 0.0,
        retries: 0,
        status: TieredPeeringSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_peering_summary("code-1", "name-131");
        assert_eq!(item.status, TieredPeeringSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_tiered_peering_summary("code-1", "name-131");
        if item.move_to(TieredPeeringSummaryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredPeeringSummaryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_peering_summary("code-1", "name-131");
        item.status = TieredPeeringSummaryStatus::Cancelled;
        assert!(
            item.move_to(TieredPeeringSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
