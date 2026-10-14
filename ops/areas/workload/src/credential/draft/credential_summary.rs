use super::*;

use acme_kit::text;

/// Where a credential summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CredentialSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for CredentialSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSummary {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub sequence: i64,
    pub ceiling: f64,
    pub port: i64,
    pub locale: String,
    pub reviewer: String,
    pub cipher: String,
    pub status: CredentialSummaryStatus,
}

impl CredentialSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialSummaryStatus) -> bool {
        match self.status {
            CredentialSummaryStatus::Idle => {
                match next {
                    CredentialSummaryStatus::Warming | CredentialSummaryStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialSummaryStatus::Warming => {
                return next == CredentialSummaryStatus::Ready || next == CredentialSummaryStatus::Draining;
            }
            CredentialSummaryStatus::Ready => {
                return next == CredentialSummaryStatus::Draining;
            }
            CredentialSummaryStatus::Draining => {
                return next == CredentialSummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential summary {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.port as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialSummary {
        CredentialSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            sequence: self.sequence,
            ceiling: self.ceiling,
            port: self.port,
            locale: self.locale.clone(),
            reviewer: self.reviewer.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a credential summary in status idle.
pub fn new_credential_summary(code: &str, label: &str) -> CredentialSummary {
    CredentialSummary {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        sequence: 0,
        ceiling: 0.0,
        port: 0,
        locale: "".to_string(),
        reviewer: "".to_string(),
        cipher: "".to_string(),
        status: CredentialSummaryStatus::Idle,
    }
}
