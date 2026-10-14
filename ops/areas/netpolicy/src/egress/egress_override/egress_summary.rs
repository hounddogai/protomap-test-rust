use super::*;

use acme_kit::text;

/// Where a egress summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress summary as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummary {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub expires_at: i64,
    pub burst: i64,
    pub description: String,
    pub locale: String,
    pub approved: bool,
    pub status: EgressSummaryStatus,
}

impl EgressSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressSummaryStatus) -> bool {
        match self.status {
            EgressSummaryStatus::Created => {
                match next {
                    EgressSummaryStatus::Validated | EgressSummaryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressSummaryStatus::Validated => {
                match next {
                    EgressSummaryStatus::Published | EgressSummaryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressSummaryStatus::Published => {
                return next == EgressSummaryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "egress summary", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressSummary {
        EgressSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            expires_at: self.expires_at,
            burst: self.burst,
            description: self.description.clone(),
            locale: self.locale.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a egress summary in status created.
pub fn new_egress_summary(code: &str, title: &str) -> EgressSummary {
    EgressSummary {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        expires_at: 0,
        burst: 0,
        description: "".to_string(),
        locale: "".to_string(),
        approved: false,
        status: EgressSummaryStatus::Created,
    }
}
