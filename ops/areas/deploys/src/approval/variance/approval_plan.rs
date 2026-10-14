use super::*;

use acme_kit::text;

/// Where a approval plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ApprovalPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ApprovalPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPlan {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub archived: bool,
    pub country: String,
    pub reviewer: String,
    pub channel: String,
    pub updated_at: i64,
    pub label: String,
    pub status: ApprovalPlanStatus,
}

impl ApprovalPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalPlanStatus) -> bool {
        match self.status {
            ApprovalPlanStatus::Requested => {
                return next == ApprovalPlanStatus::Granted || next == ApprovalPlanStatus::Expired;
            }
            ApprovalPlanStatus::Granted => {
                return next == ApprovalPlanStatus::Revoked || next == ApprovalPlanStatus::Expired;
            }
            ApprovalPlanStatus::Revoked => {
                match next {
                    ApprovalPlanStatus::Expired => {
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

    pub fn move_to(&mut self, next: ApprovalPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval plan", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> ApprovalPlan {
        ApprovalPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            archived: self.archived,
            country: self.country.clone(),
            reviewer: self.reviewer.clone(),
            channel: self.channel.clone(),
            updated_at: self.updated_at,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a approval plan in status requested.
pub fn new_approval_plan(code: &str, name: &str) -> ApprovalPlan {
    ApprovalPlan {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        archived: false,
        country: "".to_string(),
        reviewer: "".to_string(),
        channel: "".to_string(),
        updated_at: 0,
        label: "".to_string(),
        status: ApprovalPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_approval_plan("code-327", "name-585");
        assert_eq!(item.status, ApprovalPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_approval_plan("code-327", "name-585");
        if item.move_to(ApprovalPlanStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalPlanStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_approval_plan("code-327", "name-585");
        item.status = ApprovalPlanStatus::Expired;
        assert!(
            item.move_to(ApprovalPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
