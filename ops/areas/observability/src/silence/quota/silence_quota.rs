use super::*;

use acme_kit::text;

/// Where a silence quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SilenceQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SilenceQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence quota as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuota {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub created_at: i64,
    pub unit: String,
    pub rate: f64,
    pub floor_value: f64,
    pub cipher: String,
    pub country: String,
    pub amount: f64,
    pub ceiling: f64,
    pub status: SilenceQuotaStatus,
}

impl SilenceQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceQuotaStatus) -> bool {
        match self.status {
            SilenceQuotaStatus::Requested => {
                match next {
                    SilenceQuotaStatus::Granted | SilenceQuotaStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceQuotaStatus::Granted => {
                return next == SilenceQuotaStatus::Revoked || next == SilenceQuotaStatus::Expired;
            }
            SilenceQuotaStatus::Revoked => {
                return next == SilenceQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence quota {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.floor_value - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceQuota {
        SilenceQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            created_at: self.created_at,
            unit: self.unit.clone(),
            rate: self.rate,
            floor_value: self.floor_value,
            cipher: self.cipher.clone(),
            country: self.country.clone(),
            amount: self.amount,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a silence quota in status requested.
pub fn new_silence_quota(code: &str, label: &str) -> SilenceQuota {
    SilenceQuota {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        created_at: 0,
        unit: "".to_string(),
        rate: 0.0,
        floor_value: 0.0,
        cipher: "".to_string(),
        country: "".to_string(),
        amount: 0.0,
        ceiling: 0.0,
        status: SilenceQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_silence_quota("code-255", "label-531");
        assert_eq!(item.status, SilenceQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_silence_quota("code-255", "label-531");
        if item.move_to(SilenceQuotaStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceQuotaStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_silence_quota("code-255", "label-531");
        item.status = SilenceQuotaStatus::Expired;
        assert!(
            item.move_to(SilenceQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
