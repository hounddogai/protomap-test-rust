use super::*;

use acme_kit::text;

/// Where a tiered limit estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredLimitEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredLimitEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredLimitEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered limit estimate as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitEstimate {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub limit: i64,
    pub updated_at: i64,
    pub tenant_id: String,
    pub status: TieredLimitEstimateStatus,
}

impl TieredLimitEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredLimitEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredLimitEstimateStatus) -> bool {
        match self.status {
            TieredLimitEstimateStatus::Requested => {
                return next == TieredLimitEstimateStatus::Revoked;
            }
            TieredLimitEstimateStatus::Granted => {
                return next == TieredLimitEstimateStatus::Revoked || next == TieredLimitEstimateStatus::Expired;
            }
            TieredLimitEstimateStatus::Revoked => {
                match next {
                    TieredLimitEstimateStatus::Expired => {
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

    pub fn move_to(&mut self, next: TieredLimitEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered limit estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.updated_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredLimitEstimate {
        TieredLimitEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            limit: self.limit,
            updated_at: self.updated_at,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered limit estimate in status requested.
pub fn new_tiered_limit_estimate(code: &str, label: &str) -> TieredLimitEstimate {
    TieredLimitEstimate {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        limit: 0,
        updated_at: 0,
        tenant_id: "".to_string(),
        status: TieredLimitEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_limit_estimate("code-389", "label-881");
        assert_eq!(item.status, TieredLimitEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_tiered_limit_estimate("code-389", "label-881");
        if item.move_to(TieredLimitEstimateStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredLimitEstimateStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_limit_estimate("code-389", "label-881");
        item.status = TieredLimitEstimateStatus::Expired;
        assert!(
            item.move_to(TieredLimitEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
