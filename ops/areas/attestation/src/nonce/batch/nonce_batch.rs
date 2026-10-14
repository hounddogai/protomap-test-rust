use super::*;

use acme_kit::text;

/// Where a nonce batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl NonceBatchStatus {
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

impl std::fmt::Display for NonceBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce batch as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceBatch {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub duration_minutes: i64,
    pub host: String,
    pub rotation_days: i64,
    pub cluster: String,
    pub channel: String,
    pub minimum: f64,
    pub status: NonceBatchStatus,
}

impl NonceBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceBatchStatus) -> bool {
        match self.status {
            NonceBatchStatus::Draft => {
                match next {
                    NonceBatchStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceBatchStatus::Pending => {
                match next {
                    NonceBatchStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceBatchStatus::Active => {
                match next {
                    NonceBatchStatus::Suspended | NonceBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceBatchStatus::Suspended => {
                match next {
                    NonceBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: NonceBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceBatch {
        NonceBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            duration_minutes: self.duration_minutes,
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            cluster: self.cluster.clone(),
            channel: self.channel.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a nonce batch in status draft.
pub fn new_nonce_batch(code: &str, label: &str) -> NonceBatch {
    NonceBatch {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        duration_minutes: 0,
        host: "".to_string(),
        rotation_days: 0,
        cluster: "".to_string(),
        channel: "".to_string(),
        minimum: 0.0,
        status: NonceBatchStatus::Draft,
    }
}
