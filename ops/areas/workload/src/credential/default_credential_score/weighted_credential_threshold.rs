use super::*;

use acme_kit::text;

/// Where a weighted credential threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCredentialThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedCredentialThresholdStatus {
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

impl std::fmt::Display for WeightedCredentialThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted credential threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialThreshold {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub sequence: i64,
    pub floor_value: f64,
    pub batch_size: i64,
    pub cipher: String,
    pub version: i64,
    pub segment: String,
    pub status: WeightedCredentialThresholdStatus,
}

impl WeightedCredentialThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCredentialThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCredentialThresholdStatus) -> bool {
        match self.status {
            WeightedCredentialThresholdStatus::Draft => {
                match next {
                    WeightedCredentialThresholdStatus::Pending | WeightedCredentialThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialThresholdStatus::Pending => {
                return next == WeightedCredentialThresholdStatus::Closed;
            }
            WeightedCredentialThresholdStatus::Active => {
                match next {
                    WeightedCredentialThresholdStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialThresholdStatus::Suspended => {
                return next == WeightedCredentialThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedCredentialThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted credential threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.floor_value - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedCredentialThreshold {
        WeightedCredentialThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            sequence: self.sequence,
            floor_value: self.floor_value,
            batch_size: self.batch_size,
            cipher: self.cipher.clone(),
            version: self.version,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted credential threshold in status draft.
pub fn new_weighted_credential_threshold(code: &str, title: &str) -> WeightedCredentialThreshold {
    WeightedCredentialThreshold {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        sequence: 0,
        floor_value: 0.0,
        batch_size: 0,
        cipher: "".to_string(),
        version: 0,
        segment: "".to_string(),
        status: WeightedCredentialThresholdStatus::Draft,
    }
}
