use super::*;

use acme_kit::text;

/// Where a rolling action item summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingActionItemSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingActionItemSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingActionItemSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling action item summary as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingActionItemSummary {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub ceiling: f64,
    pub country: String,
    pub priority: i64,
    pub margin: f64,
    pub status: RollingActionItemSummaryStatus,
}

impl RollingActionItemSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingActionItemSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingActionItemSummaryStatus) -> bool {
        match self.status {
            RollingActionItemSummaryStatus::Requested => {
                return next == RollingActionItemSummaryStatus::Revoked;
            }
            RollingActionItemSummaryStatus::Granted => {
                match next {
                    RollingActionItemSummaryStatus::Revoked | RollingActionItemSummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingActionItemSummaryStatus::Revoked => {
                match next {
                    RollingActionItemSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollingActionItemSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling action item summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.starts_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingActionItemSummary {
        RollingActionItemSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            ceiling: self.ceiling,
            country: self.country.clone(),
            priority: self.priority,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling action item summary in status requested.
pub fn new_rolling_action_item_summary(code: &str, title: &str) -> RollingActionItemSummary {
    RollingActionItemSummary {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        ceiling: 0.0,
        country: "".to_string(),
        priority: 0,
        margin: 0.0,
        status: RollingActionItemSummaryStatus::Requested,
    }
}
