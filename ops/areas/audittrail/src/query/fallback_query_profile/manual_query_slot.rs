use super::*;

use acme_kit::text;

/// Where a manual query slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualQuerySlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualQuerySlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualQuerySlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual query slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuerySlot {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub expires_at: i64,
    pub fingerprint: String,
    pub status: ManualQuerySlotStatus,
}

impl ManualQuerySlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualQuerySlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualQuerySlotStatus) -> bool {
        match self.status {
            ManualQuerySlotStatus::Requested => {
                match next {
                    ManualQuerySlotStatus::Revoked | ManualQuerySlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualQuerySlotStatus::Granted => {
                match next {
                    ManualQuerySlotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualQuerySlotStatus::Revoked => {
                match next {
                    ManualQuerySlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualQuerySlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual query slot {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualQuerySlot {
        ManualQuerySlot {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            expires_at: self.expires_at,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual query slot in status requested.
pub fn new_manual_query_slot(code: &str, title: &str) -> ManualQuerySlot {
    ManualQuerySlot {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        expires_at: 0,
        fingerprint: "".to_string(),
        status: ManualQuerySlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_query_slot("code-15", "title-452");
        assert_eq!(item.status, ManualQuerySlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_manual_query_slot("code-15", "title-452");
        if item.move_to(ManualQuerySlotStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualQuerySlotStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_query_slot("code-15", "title-452");
        item.status = ManualQuerySlotStatus::Expired;
        assert!(
            item.move_to(ManualQuerySlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
