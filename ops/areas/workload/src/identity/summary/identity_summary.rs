use super::*;

use acme_kit::text;

/// Where a identity summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentitySummaryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentitySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentitySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One identity summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentitySummary {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub rotation_days: i64,
    pub minimum: f64,
    pub status: IdentitySummaryStatus,
}

impl IdentitySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentitySummaryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentitySummaryStatus) -> bool {
        match self.status {
            IdentitySummaryStatus::Requested => {
                match next {
                    IdentitySummaryStatus::Revoked | IdentitySummaryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentitySummaryStatus::Granted => {
                return next == IdentitySummaryStatus::Revoked || next == IdentitySummaryStatus::Expired;
            }
            IdentitySummaryStatus::Revoked => {
                return next == IdentitySummaryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentitySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentitySummary {
        IdentitySummary {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            rotation_days: self.rotation_days,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a identity summary in status requested.
pub fn new_identity_summary(code: &str, label: &str) -> IdentitySummary {
    IdentitySummary {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        rotation_days: 0,
        minimum: 0.0,
        status: IdentitySummaryStatus::Requested,
    }
}
