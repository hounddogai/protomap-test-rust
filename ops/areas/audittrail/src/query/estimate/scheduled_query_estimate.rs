use super::*;

use acme_kit::text;

/// Where a scheduled query estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQueryEstimateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledQueryEstimateStatus {
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

impl std::fmt::Display for ScheduledQueryEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled query estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQueryEstimate {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub region: String,
    pub cluster: String,
    pub cipher: String,
    pub status: ScheduledQueryEstimateStatus,
}

impl ScheduledQueryEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQueryEstimateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQueryEstimateStatus) -> bool {
        match self.status {
            ScheduledQueryEstimateStatus::Draft => {
                return next == ScheduledQueryEstimateStatus::Pending || next == ScheduledQueryEstimateStatus::Active;
            }
            ScheduledQueryEstimateStatus::Pending => {
                match next {
                    ScheduledQueryEstimateStatus::Active | ScheduledQueryEstimateStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryEstimateStatus::Active => {
                match next {
                    ScheduledQueryEstimateStatus::Suspended | ScheduledQueryEstimateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQueryEstimateStatus::Suspended => {
                return next == ScheduledQueryEstimateStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledQueryEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled query estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled query estimate", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> ScheduledQueryEstimate {
        ScheduledQueryEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            region: self.region.clone(),
            cluster: self.cluster.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a scheduled query estimate in status draft.
pub fn new_scheduled_query_estimate(code: &str, title: &str) -> ScheduledQueryEstimate {
    ScheduledQueryEstimate {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        region: "".to_string(),
        cluster: "".to_string(),
        cipher: "".to_string(),
        status: ScheduledQueryEstimateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_query_estimate("code-664", "title-644");
        assert_eq!(item.status, ScheduledQueryEstimateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_query_estimate("code-664", "title-644");
        if item.move_to(ScheduledQueryEstimateStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQueryEstimateStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_query_estimate("code-664", "title-644");
        item.status = ScheduledQueryEstimateStatus::Closed;
        assert!(
            item.move_to(ScheduledQueryEstimateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
