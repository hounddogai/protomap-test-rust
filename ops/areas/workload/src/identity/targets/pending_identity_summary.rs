use super::*;

use acme_kit::text;

/// Where a pending identity summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingIdentitySummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingIdentitySummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingIdentitySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending identity summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentitySummary {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub external_ref: String,
    pub contact_email: String,
    pub key_id: String,
    pub sequence: i64,
    pub status: PendingIdentitySummaryStatus,
}

impl PendingIdentitySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingIdentitySummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingIdentitySummaryStatus) -> bool {
        match self.status {
            PendingIdentitySummaryStatus::Created => {
                match next {
                    PendingIdentitySummaryStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingIdentitySummaryStatus::Validated => {
                return next == PendingIdentitySummaryStatus::Published;
            }
            PendingIdentitySummaryStatus::Published => {
                match next {
                    PendingIdentitySummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingIdentitySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending identity summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending identity summary", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.sequence as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingIdentitySummary {
        PendingIdentitySummary {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            contact_email: self.contact_email.clone(),
            key_id: self.key_id.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending identity summary in status created.
pub fn new_pending_identity_summary(code: &str, title: &str) -> PendingIdentitySummary {
    PendingIdentitySummary {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        contact_email: "".to_string(),
        key_id: "".to_string(),
        sequence: 0,
        status: PendingIdentitySummaryStatus::Created,
    }
}
