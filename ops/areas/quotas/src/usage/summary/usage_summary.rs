use super::*;

use acme_kit::text;

/// Where a usage summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageSummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummary {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub rate: f64,
    pub limit: i64,
    pub starts_at: i64,
    pub key_id: String,
    pub attempts: i64,
    pub status: UsageSummaryStatus,
}

impl UsageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageSummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageSummaryStatus) -> bool {
        match self.status {
            UsageSummaryStatus::Requested => {
                match next {
                    UsageSummaryStatus::Granted | UsageSummaryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageSummaryStatus::Granted => {
                match next {
                    UsageSummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageSummaryStatus::Revoked => {
                match next {
                    UsageSummaryStatus::Expired => {
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

    pub fn move_to(&mut self, next: UsageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage summary {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ceiling - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageSummary {
        UsageSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            rate: self.rate,
            limit: self.limit,
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a usage summary in status requested.
pub fn new_usage_summary(code: &str, name: &str) -> UsageSummary {
    UsageSummary {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        rate: 0.0,
        limit: 0,
        starts_at: 0,
        key_id: "".to_string(),
        attempts: 0,
        status: UsageSummaryStatus::Requested,
    }
}
