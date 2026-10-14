use super::*;

use acme_kit::text;

/// Where a identity quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl IdentityQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for IdentityQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityQuota {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub approved: bool,
    pub source: String,
    pub quantity: i64,
    pub status: IdentityQuotaStatus,
}

impl IdentityQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityQuotaStatus) -> bool {
        match self.status {
            IdentityQuotaStatus::New => {
                return next == IdentityQuotaStatus::Queued || next == IdentityQuotaStatus::Running;
            }
            IdentityQuotaStatus::Queued => {
                return next == IdentityQuotaStatus::Done;
            }
            IdentityQuotaStatus::Running => {
                match next {
                    IdentityQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityQuotaStatus::Done => {
                match next {
                    IdentityQuotaStatus::Failed => {
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

    pub fn move_to(&mut self, next: IdentityQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> IdentityQuota {
        IdentityQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            approved: self.approved,
            source: self.source.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a identity quota in status new.
pub fn new_identity_quota(code: &str, name: &str) -> IdentityQuota {
    IdentityQuota {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        approved: false,
        source: "".to_string(),
        quantity: 0,
        status: IdentityQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_identity_quota("code-79", "name-734");
        assert_eq!(item.status, IdentityQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_identity_quota("code-79", "name-734");
        if item.move_to(IdentityQuotaStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityQuotaStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_identity_quota("code-79", "name-734");
        item.status = IdentityQuotaStatus::Failed;
        assert!(item.move_to(IdentityQuotaStatus::New).is_err(), "expected move to fail");
    }
}
