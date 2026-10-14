use super::*;

use acme_kit::text;

/// Where a nonce digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl NonceDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for NonceDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceDigest {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub unit: String,
    pub namespace: String,
    pub ratio: f64,
    pub quantity: i64,
    pub visible: bool,
    pub weight: f64,
    pub maximum: f64,
    pub status: NonceDigestStatus,
}

impl NonceDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceDigestStatus) -> bool {
        match self.status {
            NonceDigestStatus::Planned => {
                match next {
                    NonceDigestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceDigestStatus::Scheduled => {
                match next {
                    NonceDigestStatus::InProgress | NonceDigestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceDigestStatus::InProgress => {
                match next {
                    NonceDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceDigestStatus::Completed => {
                return next == NonceDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.maximum * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceDigest {
        NonceDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            unit: self.unit.clone(),
            namespace: self.namespace.clone(),
            ratio: self.ratio,
            quantity: self.quantity,
            visible: self.visible,
            weight: self.weight,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a nonce digest in status planned.
pub fn new_nonce_digest(code: &str, label: &str) -> NonceDigest {
    NonceDigest {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        unit: "".to_string(),
        namespace: "".to_string(),
        ratio: 0.0,
        quantity: 0,
        visible: false,
        weight: 0.0,
        maximum: 0.0,
        status: NonceDigestStatus::Planned,
    }
}
