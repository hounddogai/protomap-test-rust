use super::*;

use acme_kit::text;

/// Where a silence summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SilenceSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SilenceSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSummary {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub starts_at: i64,
    pub budget: f64,
    pub contact_email: String,
    pub status: SilenceSummaryStatus,
}

impl SilenceSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceSummaryStatus) -> bool {
        match self.status {
            SilenceSummaryStatus::Created => {
                return next == SilenceSummaryStatus::Validated || next == SilenceSummaryStatus::Retired;
            }
            SilenceSummaryStatus::Validated => {
                return next == SilenceSummaryStatus::Retired;
            }
            SilenceSummaryStatus::Published => {
                return next == SilenceSummaryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence summary {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.starts_at as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceSummary {
        SilenceSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            starts_at: self.starts_at,
            budget: self.budget,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence summary in status created.
pub fn new_silence_summary(code: &str, label: &str) -> SilenceSummary {
    SilenceSummary {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        starts_at: 0,
        budget: 0.0,
        contact_email: "".to_string(),
        status: SilenceSummaryStatus::Created,
    }
}
