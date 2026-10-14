use super::*;

use acme_kit::text;

/// Where a legacy tenant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTenantAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyTenantAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LegacyTenantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy tenant agreement as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAgreement {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub currency: String,
    pub approved: bool,
    pub namespace: String,
    pub maximum: f64,
    pub rotation_days: i64,
    pub expires_at: i64,
    pub batch_size: i64,
    pub status: LegacyTenantAgreementStatus,
}

impl LegacyTenantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTenantAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTenantAgreementStatus) -> bool {
        match self.status {
            LegacyTenantAgreementStatus::Planned => {
                return next == LegacyTenantAgreementStatus::InProgress;
            }
            LegacyTenantAgreementStatus::Scheduled => {
                match next {
                    LegacyTenantAgreementStatus::InProgress | LegacyTenantAgreementStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTenantAgreementStatus::InProgress => {
                return next == LegacyTenantAgreementStatus::Completed
                    || next == LegacyTenantAgreementStatus::Cancelled;
            }
            LegacyTenantAgreementStatus::Completed => {
                match next {
                    LegacyTenantAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyTenantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy tenant agreement {} cannot move from {} to {}",
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
        format!("{} {}", "legacy tenant agreement", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.expires_at as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTenantAgreement {
        LegacyTenantAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            currency: self.currency.clone(),
            approved: self.approved,
            namespace: self.namespace.clone(),
            maximum: self.maximum,
            rotation_days: self.rotation_days,
            expires_at: self.expires_at,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy tenant agreement in status planned.
pub fn new_legacy_tenant_agreement(code: &str, title: &str) -> LegacyTenantAgreement {
    LegacyTenantAgreement {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        currency: "".to_string(),
        approved: false,
        namespace: "".to_string(),
        maximum: 0.0,
        rotation_days: 0,
        expires_at: 0,
        batch_size: 0,
        status: LegacyTenantAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_tenant_agreement("code-567", "title-703");
        assert_eq!(item.status, LegacyTenantAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_legacy_tenant_agreement("code-567", "title-703");
        if item.move_to(LegacyTenantAgreementStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTenantAgreementStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_tenant_agreement("code-567", "title-703");
        item.status = LegacyTenantAgreementStatus::Cancelled;
        assert!(
            item.move_to(LegacyTenantAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
