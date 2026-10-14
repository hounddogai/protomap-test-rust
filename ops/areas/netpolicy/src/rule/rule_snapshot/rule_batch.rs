use super::*;

use acme_kit::text;

/// Where a rule batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RuleBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RuleBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleBatch {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub duration_minutes: i64,
    pub checksum: String,
    pub expires_at: i64,
    pub status: RuleBatchStatus,
}

impl RuleBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleBatchStatus) -> bool {
        match self.status {
            RuleBatchStatus::Requested => {
                match next {
                    RuleBatchStatus::Granted | RuleBatchStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleBatchStatus::Granted => {
                match next {
                    RuleBatchStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleBatchStatus::Revoked => {
                return next == RuleBatchStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule batch {} cannot move from {} to {}",
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
        format!("{} {}", "rule batch", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.expires_at as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleBatch {
        RuleBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            duration_minutes: self.duration_minutes,
            checksum: self.checksum.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rule batch in status requested.
pub fn new_rule_batch(code: &str, name: &str) -> RuleBatch {
    RuleBatch {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        duration_minutes: 0,
        checksum: "".to_string(),
        expires_at: 0,
        status: RuleBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rule_batch("code-472", "name-858");
        assert_eq!(item.status, RuleBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rule_batch("code-472", "name-858");
        if item.move_to(RuleBatchStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleBatchStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rule_batch("code-472", "name-858");
        item.status = RuleBatchStatus::Expired;
        assert!(
            item.move_to(RuleBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
