use super::*;

use acme_kit::text;

/// Where a lenient tenant quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTenantQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientTenantQuotaStatus {
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

impl std::fmt::Display for LenientTenantQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient tenant quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantQuota {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub locale: String,
    pub host: String,
    pub pinned: bool,
    pub duration_minutes: i64,
    pub country: String,
    pub cipher: String,
    pub amount: f64,
    pub title: String,
    pub status: LenientTenantQuotaStatus,
}

impl LenientTenantQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTenantQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTenantQuotaStatus) -> bool {
        match self.status {
            LenientTenantQuotaStatus::Draft => {
                match next {
                    LenientTenantQuotaStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTenantQuotaStatus::Pending => {
                return next == LenientTenantQuotaStatus::Active;
            }
            LenientTenantQuotaStatus::Active => {
                return next == LenientTenantQuotaStatus::Suspended || next == LenientTenantQuotaStatus::Closed;
            }
            LenientTenantQuotaStatus::Suspended => {
                match next {
                    LenientTenantQuotaStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientTenantQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient tenant quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTenantQuota {
        LenientTenantQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            locale: self.locale.clone(),
            host: self.host.clone(),
            pinned: self.pinned,
            duration_minutes: self.duration_minutes,
            country: self.country.clone(),
            cipher: self.cipher.clone(),
            amount: self.amount,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a lenient tenant quota in status draft.
pub fn new_lenient_tenant_quota(code: &str, name: &str) -> LenientTenantQuota {
    LenientTenantQuota {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        locale: "".to_string(),
        host: "".to_string(),
        pinned: false,
        duration_minutes: 0,
        country: "".to_string(),
        cipher: "".to_string(),
        amount: 0.0,
        title: "".to_string(),
        status: LenientTenantQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_tenant_quota("code-672", "name-742");
        assert_eq!(item.status, LenientTenantQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_tenant_quota("code-672", "name-742");
        if item.move_to(LenientTenantQuotaStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTenantQuotaStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_tenant_quota("code-672", "name-742");
        item.status = LenientTenantQuotaStatus::Closed;
        assert!(
            item.move_to(LenientTenantQuotaStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
