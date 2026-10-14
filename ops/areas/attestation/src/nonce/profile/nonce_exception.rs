use super::*;

use acme_kit::text;

/// Where a nonce exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl NonceExceptionStatus {
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

impl std::fmt::Display for NonceExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce exception as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceException {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub capacity: i64,
    pub created_at: i64,
    pub grace_days: i64,
    pub burst: i64,
    pub floor_value: f64,
    pub reviewer: String,
    pub label: String,
    pub status: NonceExceptionStatus,
}

impl NonceException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceExceptionStatus) -> bool {
        match self.status {
            NonceExceptionStatus::Draft => {
                return next == NonceExceptionStatus::Pending || next == NonceExceptionStatus::Active;
            }
            NonceExceptionStatus::Pending => {
                match next {
                    NonceExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceExceptionStatus::Active => {
                return next == NonceExceptionStatus::Suspended || next == NonceExceptionStatus::Closed;
            }
            NonceExceptionStatus::Suspended => {
                return next == NonceExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "nonce exception", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceException {
        NonceException {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            capacity: self.capacity,
            created_at: self.created_at,
            grace_days: self.grace_days,
            burst: self.burst,
            floor_value: self.floor_value,
            reviewer: self.reviewer.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a nonce exception in status draft.
pub fn new_nonce_exception(code: &str, title: &str) -> NonceException {
    NonceException {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        capacity: 0,
        created_at: 0,
        grace_days: 0,
        burst: 0,
        floor_value: 0.0,
        reviewer: "".to_string(),
        label: "".to_string(),
        status: NonceExceptionStatus::Draft,
    }
}
