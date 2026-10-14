use super::*;

use acme_kit::text;

/// Where a overage agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OverageAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for OverageAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageAgreement {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub port: i64,
    pub ratio: f64,
    pub visible: bool,
    pub attempts: i64,
    pub created_at: i64,
    pub batch_size: i64,
    pub locale: String,
    pub quantity: i64,
    pub status: OverageAgreementStatus,
}

impl OverageAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageAgreementStatus) -> bool {
        match self.status {
            OverageAgreementStatus::Open => {
                match next {
                    OverageAgreementStatus::InReview | OverageAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageAgreementStatus::InReview => {
                match next {
                    OverageAgreementStatus::Resolved | OverageAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageAgreementStatus::Resolved => {
                return next == OverageAgreementStatus::Reopened || next == OverageAgreementStatus::Archived;
            }
            OverageAgreementStatus::Reopened => {
                return next == OverageAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage agreement {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.quantity as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageAgreement {
        OverageAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            ratio: self.ratio,
            visible: self.visible,
            attempts: self.attempts,
            created_at: self.created_at,
            batch_size: self.batch_size,
            locale: self.locale.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a overage agreement in status open.
pub fn new_overage_agreement(code: &str, title: &str) -> OverageAgreement {
    OverageAgreement {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        port: 0,
        ratio: 0.0,
        visible: false,
        attempts: 0,
        created_at: 0,
        batch_size: 0,
        locale: "".to_string(),
        quantity: 0,
        status: OverageAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_overage_agreement("code-681", "title-35");
        assert_eq!(item.status, OverageAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_overage_agreement("code-681", "title-35");
        if item.move_to(OverageAgreementStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageAgreementStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_overage_agreement("code-681", "title-35");
        item.status = OverageAgreementStatus::Archived;
        assert!(
            item.move_to(OverageAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
