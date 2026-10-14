use super::*;

use acme_kit::text;

/// Where a tiered usage summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredUsageSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredUsageSummaryStatus {
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

impl std::fmt::Display for TieredUsageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered usage summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageSummary {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub limit: i64,
    pub created_at: i64,
    pub issuer: String,
    pub cipher: String,
    pub status: TieredUsageSummaryStatus,
}

impl TieredUsageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredUsageSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredUsageSummaryStatus) -> bool {
        match self.status {
            TieredUsageSummaryStatus::Planned => {
                return next == TieredUsageSummaryStatus::Completed || next == TieredUsageSummaryStatus::Cancelled;
            }
            TieredUsageSummaryStatus::Scheduled => {
                return next == TieredUsageSummaryStatus::InProgress || next == TieredUsageSummaryStatus::Completed;
            }
            TieredUsageSummaryStatus::InProgress => {
                return next == TieredUsageSummaryStatus::Completed || next == TieredUsageSummaryStatus::Cancelled;
            }
            TieredUsageSummaryStatus::Completed => {
                match next {
                    TieredUsageSummaryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredUsageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered usage summary {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.limit as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredUsageSummary {
        TieredUsageSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            limit: self.limit,
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered usage summary in status planned.
pub fn new_tiered_usage_summary(code: &str, name: &str) -> TieredUsageSummary {
    TieredUsageSummary {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        limit: 0,
        created_at: 0,
        issuer: "".to_string(),
        cipher: "".to_string(),
        status: TieredUsageSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_usage_summary("code-175", "name-920");
        assert_eq!(item.status, TieredUsageSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_tiered_usage_summary("code-175", "name-920");
        if item.move_to(TieredUsageSummaryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredUsageSummaryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_usage_summary("code-175", "name-920");
        item.status = TieredUsageSummaryStatus::Cancelled;
        assert!(
            item.move_to(TieredUsageSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
