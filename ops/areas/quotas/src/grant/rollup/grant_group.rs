use super::*;

use acme_kit::text;

/// Where a grant group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GrantGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GrantGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantGroup {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub category: String,
    pub tenant_id: String,
    pub approved: bool,
    pub grace_days: i64,
    pub status: GrantGroupStatus,
}

impl GrantGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantGroupStatus) -> bool {
        match self.status {
            GrantGroupStatus::Requested => {
                match next {
                    GrantGroupStatus::Granted | GrantGroupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantGroupStatus::Granted => {
                match next {
                    GrantGroupStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantGroupStatus::Revoked => {
                return next == GrantGroupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> GrantGroup {
        GrantGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            category: self.category.clone(),
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a grant group in status requested.
pub fn new_grant_group(code: &str, name: &str) -> GrantGroup {
    GrantGroup {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        category: "".to_string(),
        tenant_id: "".to_string(),
        approved: false,
        grace_days: 0,
        status: GrantGroupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_grant_group("code-983", "name-481");
        assert_eq!(item.status, GrantGroupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_grant_group("code-983", "name-481");
        if item.move_to(GrantGroupStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantGroupStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_grant_group("code-983", "name-481");
        item.status = GrantGroupStatus::Expired;
        assert!(
            item.move_to(GrantGroupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
