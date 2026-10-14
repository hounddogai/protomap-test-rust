use super::*;

use acme_kit::text;

/// Where a credential plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CredentialPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CredentialPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialPlan {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub tenant_id: String,
    pub window_days: i64,
    pub status: CredentialPlanStatus,
}

impl CredentialPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialPlanStatus) -> bool {
        match self.status {
            CredentialPlanStatus::Requested => {
                match next {
                    CredentialPlanStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialPlanStatus::Granted => {
                match next {
                    CredentialPlanStatus::Revoked | CredentialPlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialPlanStatus::Revoked => {
                match next {
                    CredentialPlanStatus::Expired => {
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

    pub fn move_to(&mut self, next: CredentialPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> CredentialPlan {
        CredentialPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            tenant_id: self.tenant_id.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a credential plan in status requested.
pub fn new_credential_plan(code: &str, title: &str) -> CredentialPlan {
    CredentialPlan {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        tenant_id: "".to_string(),
        window_days: 0,
        status: CredentialPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_credential_plan("code-645", "title-704");
        assert_eq!(item.status, CredentialPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_credential_plan("code-645", "title-704");
        if item.move_to(CredentialPlanStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialPlanStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_credential_plan("code-645", "title-704");
        item.status = CredentialPlanStatus::Expired;
        assert!(
            item.move_to(CredentialPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
