use super::*;

use acme_kit::text;

/// Where a primary limit audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryLimitAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryLimitAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryLimitAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary limit audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitAudit {
    pub code: String,
    pub title: String,
    pub country: String,
    pub notes: String,
    pub cost: f64,
    pub cluster: String,
    pub status: PrimaryLimitAuditStatus,
}

impl PrimaryLimitAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryLimitAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryLimitAuditStatus) -> bool {
        match self.status {
            PrimaryLimitAuditStatus::Requested => {
                match next {
                    PrimaryLimitAuditStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryLimitAuditStatus::Granted => {
                match next {
                    PrimaryLimitAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryLimitAuditStatus::Revoked => {
                return next == PrimaryLimitAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryLimitAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary limit audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary limit audit", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> PrimaryLimitAudit {
        PrimaryLimitAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            notes: self.notes.clone(),
            cost: self.cost,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary limit audit in status requested.
pub fn new_primary_limit_audit(code: &str, title: &str) -> PrimaryLimitAudit {
    PrimaryLimitAudit {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        notes: "".to_string(),
        cost: 0.0,
        cluster: "".to_string(),
        status: PrimaryLimitAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_limit_audit("code-816", "title-424");
        assert_eq!(item.status, PrimaryLimitAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_primary_limit_audit("code-816", "title-424");
        if item.move_to(PrimaryLimitAuditStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryLimitAuditStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_limit_audit("code-816", "title-424");
        item.status = PrimaryLimitAuditStatus::Expired;
        assert!(
            item.move_to(PrimaryLimitAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
