use super::*;

use acme_kit::text;

/// Where a legacy quote batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyQuoteBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyQuoteBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyQuoteBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy quote batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteBatch {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub locale: String,
    pub batch_size: i64,
    pub scope: String,
    pub key_id: String,
    pub minimum: f64,
    pub expires_at: i64,
    pub status: LegacyQuoteBatchStatus,
}

impl LegacyQuoteBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyQuoteBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyQuoteBatchStatus) -> bool {
        match self.status {
            LegacyQuoteBatchStatus::Requested => {
                match next {
                    LegacyQuoteBatchStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyQuoteBatchStatus::Granted => {
                return next == LegacyQuoteBatchStatus::Expired;
            }
            LegacyQuoteBatchStatus::Revoked => {
                match next {
                    LegacyQuoteBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyQuoteBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy quote batch {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyQuoteBatch {
        LegacyQuoteBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            locale: self.locale.clone(),
            batch_size: self.batch_size,
            scope: self.scope.clone(),
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy quote batch in status requested.
pub fn new_legacy_quote_batch(code: &str, name: &str) -> LegacyQuoteBatch {
    LegacyQuoteBatch {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        locale: "".to_string(),
        batch_size: 0,
        scope: "".to_string(),
        key_id: "".to_string(),
        minimum: 0.0,
        expires_at: 0,
        status: LegacyQuoteBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_quote_batch("code-913", "name-259");
        assert_eq!(item.status, LegacyQuoteBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_legacy_quote_batch("code-913", "name-259");
        if item.move_to(LegacyQuoteBatchStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyQuoteBatchStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_quote_batch("code-913", "name-259");
        item.status = LegacyQuoteBatchStatus::Expired;
        assert!(
            item.move_to(LegacyQuoteBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
