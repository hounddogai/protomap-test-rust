use super::*;

use acme_kit::text;

/// Where a weighted trust domain entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTrustDomainEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedTrustDomainEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedTrustDomainEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted trust domain entry as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainEntry {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub attempts: i64,
    pub ratio: f64,
    pub ttl_seconds: i64,
    pub status: WeightedTrustDomainEntryStatus,
}

impl WeightedTrustDomainEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTrustDomainEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTrustDomainEntryStatus) -> bool {
        match self.status {
            WeightedTrustDomainEntryStatus::New => {
                match next {
                    WeightedTrustDomainEntryStatus::Running | WeightedTrustDomainEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTrustDomainEntryStatus::Queued => {
                return next == WeightedTrustDomainEntryStatus::Running;
            }
            WeightedTrustDomainEntryStatus::Running => {
                return next == WeightedTrustDomainEntryStatus::Failed;
            }
            WeightedTrustDomainEntryStatus::Done => {
                match next {
                    WeightedTrustDomainEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedTrustDomainEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted trust domain entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted trust domain entry", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ratio - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTrustDomainEntry {
        WeightedTrustDomainEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            attempts: self.attempts,
            ratio: self.ratio,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted trust domain entry in status new.
pub fn new_weighted_trust_domain_entry(code: &str, label: &str) -> WeightedTrustDomainEntry {
    WeightedTrustDomainEntry {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        attempts: 0,
        ratio: 0.0,
        ttl_seconds: 0,
        status: WeightedTrustDomainEntryStatus::New,
    }
}
