use super::*;

use acme_kit::text;

/// Where a manual credential quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualCredentialQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualCredentialQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualCredentialQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual credential quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCredentialQuota {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub channel: String,
    pub minimum: f64,
    pub status: ManualCredentialQuotaStatus,
}

impl ManualCredentialQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualCredentialQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualCredentialQuotaStatus) -> bool {
        match self.status {
            ManualCredentialQuotaStatus::Proposed => {
                match next {
                    ManualCredentialQuotaStatus::Approved | ManualCredentialQuotaStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCredentialQuotaStatus::Approved => {
                match next {
                    ManualCredentialQuotaStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCredentialQuotaStatus::Rejected => {
                return next == ManualCredentialQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualCredentialQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual credential quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual credential quota", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.minimum * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualCredentialQuota {
        ManualCredentialQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            channel: self.channel.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual credential quota in status proposed.
pub fn new_manual_credential_quota(code: &str, name: &str) -> ManualCredentialQuota {
    ManualCredentialQuota {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        channel: "".to_string(),
        minimum: 0.0,
        status: ManualCredentialQuotaStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_credential_quota("code-986", "name-177");
        assert_eq!(item.status, ManualCredentialQuotaStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_manual_credential_quota("code-986", "name-177");
        if item.move_to(ManualCredentialQuotaStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualCredentialQuotaStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_credential_quota("code-986", "name-177");
        item.status = ManualCredentialQuotaStatus::Withdrawn;
        assert!(
            item.move_to(ManualCredentialQuotaStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
