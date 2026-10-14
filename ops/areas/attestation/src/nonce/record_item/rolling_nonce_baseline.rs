use super::*;

use acme_kit::text;

/// Where a rolling nonce baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingNonceBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingNonceBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingNonceBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling nonce baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaseline {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub discount: f64,
    pub key_id: String,
    pub created_at: i64,
    pub channel: String,
    pub currency: String,
    pub status: RollingNonceBaselineStatus,
}

impl RollingNonceBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingNonceBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingNonceBaselineStatus) -> bool {
        match self.status {
            RollingNonceBaselineStatus::Created => {
                return next == RollingNonceBaselineStatus::Validated || next == RollingNonceBaselineStatus::Published;
            }
            RollingNonceBaselineStatus::Validated => {
                return next == RollingNonceBaselineStatus::Published || next == RollingNonceBaselineStatus::Retired;
            }
            RollingNonceBaselineStatus::Published => {
                return next == RollingNonceBaselineStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingNonceBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling nonce baseline {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.created_at as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingNonceBaseline {
        RollingNonceBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            discount: self.discount,
            key_id: self.key_id.clone(),
            created_at: self.created_at,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling nonce baseline in status created.
pub fn new_rolling_nonce_baseline(code: &str, title: &str) -> RollingNonceBaseline {
    RollingNonceBaseline {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        discount: 0.0,
        key_id: "".to_string(),
        created_at: 0,
        channel: "".to_string(),
        currency: "".to_string(),
        status: RollingNonceBaselineStatus::Created,
    }
}
