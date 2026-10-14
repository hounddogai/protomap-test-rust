use super::*;

use acme_kit::text;

/// Where a delegation audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DelegationAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DelegationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAudit {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub description: String,
    pub discount: f64,
    pub category: String,
    pub status: DelegationAuditStatus,
}

impl DelegationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationAuditStatus) -> bool {
        match self.status {
            DelegationAuditStatus::Requested => {
                match next {
                    DelegationAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationAuditStatus::Granted => {
                match next {
                    DelegationAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationAuditStatus::Revoked => {
                return next == DelegationAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "delegation audit", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.floor_value + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationAudit {
        DelegationAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            description: self.description.clone(),
            discount: self.discount,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a delegation audit in status requested.
pub fn new_delegation_audit(code: &str, name: &str) -> DelegationAudit {
    DelegationAudit {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        description: "".to_string(),
        discount: 0.0,
        category: "".to_string(),
        status: DelegationAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_delegation_audit("code-961", "name-759");
        assert_eq!(item.status, DelegationAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_delegation_audit("code-961", "name-759");
        if item.move_to(DelegationAuditStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationAuditStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_delegation_audit("code-961", "name-759");
        item.status = DelegationAuditStatus::Expired;
        assert!(
            item.move_to(DelegationAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
