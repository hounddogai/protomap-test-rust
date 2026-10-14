use super::*;

use acme_kit::text;

/// Where a weighted credential variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCredentialVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedCredentialVarianceStatus {
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

impl std::fmt::Display for WeightedCredentialVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted credential variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialVariance {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub key_id: String,
    pub threshold: f64,
    pub minimum: f64,
    pub sequence: i64,
    pub category: String,
    pub status: WeightedCredentialVarianceStatus,
}

impl WeightedCredentialVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCredentialVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCredentialVarianceStatus) -> bool {
        match self.status {
            WeightedCredentialVarianceStatus::Draft => {
                return next == WeightedCredentialVarianceStatus::Active;
            }
            WeightedCredentialVarianceStatus::Pending => {
                return next == WeightedCredentialVarianceStatus::Active
                    || next == WeightedCredentialVarianceStatus::Closed;
            }
            WeightedCredentialVarianceStatus::Active => {
                return next == WeightedCredentialVarianceStatus::Suspended
                    || next == WeightedCredentialVarianceStatus::Closed;
            }
            WeightedCredentialVarianceStatus::Suspended => {
                return next == WeightedCredentialVarianceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedCredentialVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted credential variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted credential variance", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        (self.minimum * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedCredentialVariance {
        WeightedCredentialVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            key_id: self.key_id.clone(),
            threshold: self.threshold,
            minimum: self.minimum,
            sequence: self.sequence,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a weighted credential variance in status draft.
pub fn new_weighted_credential_variance(code: &str, label: &str) -> WeightedCredentialVariance {
    WeightedCredentialVariance {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        key_id: "".to_string(),
        threshold: 0.0,
        minimum: 0.0,
        sequence: 0,
        category: "".to_string(),
        status: WeightedCredentialVarianceStatus::Draft,
    }
}
