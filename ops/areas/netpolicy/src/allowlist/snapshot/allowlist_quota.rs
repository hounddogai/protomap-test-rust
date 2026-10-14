use super::*;

use acme_kit::text;

/// Where a allowlist quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AllowlistQuotaStatus {
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

impl std::fmt::Display for AllowlistQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistQuota {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub batch_size: i64,
    pub fingerprint: String,
    pub ceiling: f64,
    pub region: String,
    pub retries: i64,
    pub expires_at: i64,
    pub source: String,
    pub status: AllowlistQuotaStatus,
}

impl AllowlistQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistQuotaStatus) -> bool {
        match self.status {
            AllowlistQuotaStatus::Draft => {
                match next {
                    AllowlistQuotaStatus::Pending | AllowlistQuotaStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistQuotaStatus::Pending => {
                match next {
                    AllowlistQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistQuotaStatus::Active => {
                return next == AllowlistQuotaStatus::Suspended || next == AllowlistQuotaStatus::Closed;
            }
            AllowlistQuotaStatus::Suspended => {
                match next {
                    AllowlistQuotaStatus::Closed => {
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

    pub fn move_to(&mut self, next: AllowlistQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist quota {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ceiling + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistQuota {
        AllowlistQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            batch_size: self.batch_size,
            fingerprint: self.fingerprint.clone(),
            ceiling: self.ceiling,
            region: self.region.clone(),
            retries: self.retries,
            expires_at: self.expires_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a allowlist quota in status draft.
pub fn new_allowlist_quota(code: &str, name: &str) -> AllowlistQuota {
    AllowlistQuota {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        batch_size: 0,
        fingerprint: "".to_string(),
        ceiling: 0.0,
        region: "".to_string(),
        retries: 0,
        expires_at: 0,
        source: "".to_string(),
        status: AllowlistQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_allowlist_quota("code-118", "name-11");
        assert_eq!(item.status, AllowlistQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_allowlist_quota("code-118", "name-11");
        if item.move_to(AllowlistQuotaStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistQuotaStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_allowlist_quota("code-118", "name-11");
        item.status = AllowlistQuotaStatus::Closed;
        assert!(
            item.move_to(AllowlistQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
