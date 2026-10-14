use super::*;

use acme_kit::text;

/// Where a tenant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TenantAgreementStatus {
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

impl std::fmt::Display for TenantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tenant agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAgreement {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub scope: String,
    pub burst: i64,
    pub threshold: f64,
    pub created_at: i64,
    pub name: String,
    pub budget: f64,
    pub archived: bool,
    pub status: TenantAgreementStatus,
}

impl TenantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantAgreementStatus) -> bool {
        match self.status {
            TenantAgreementStatus::Open => {
                match next {
                    TenantAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAgreementStatus::InReview => {
                match next {
                    TenantAgreementStatus::Resolved | TenantAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAgreementStatus::Resolved => {
                match next {
                    TenantAgreementStatus::Reopened | TenantAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantAgreementStatus::Reopened => {
                match next {
                    TenantAgreementStatus::Archived => {
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

    pub fn move_to(&mut self, next: TenantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant agreement {} cannot move from {} to {}",
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

    pub fn net_volume(&self) -> f64 {
        ((self.created_at as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantAgreement {
        TenantAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            scope: self.scope.clone(),
            burst: self.burst,
            threshold: self.threshold,
            created_at: self.created_at,
            name: self.name.clone(),
            budget: self.budget,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tenant agreement in status open.
pub fn new_tenant_agreement(code: &str, title: &str) -> TenantAgreement {
    TenantAgreement {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        scope: "".to_string(),
        burst: 0,
        threshold: 0.0,
        created_at: 0,
        name: "".to_string(),
        budget: 0.0,
        archived: false,
        status: TenantAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tenant_agreement("code-138", "title-402");
        assert_eq!(item.status, TenantAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_tenant_agreement("code-138", "title-402");
        if item.move_to(TenantAgreementStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TenantAgreementStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tenant_agreement("code-138", "title-402");
        item.status = TenantAgreementStatus::Archived;
        assert!(
            item.move_to(TenantAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
