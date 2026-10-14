use super::*;

use acme_kit::text;

/// Where a manual rollback profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRollbackProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualRollbackProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualRollbackProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual rollback profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRollbackProfile {
    pub code: String,
    pub title: String,
    pub region: String,
    pub visible: bool,
    pub discount: f64,
    pub archived: bool,
    pub audience: String,
    pub status: ManualRollbackProfileStatus,
}

impl ManualRollbackProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRollbackProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRollbackProfileStatus) -> bool {
        match self.status {
            ManualRollbackProfileStatus::Requested => {
                return next == ManualRollbackProfileStatus::Granted || next == ManualRollbackProfileStatus::Expired;
            }
            ManualRollbackProfileStatus::Granted => {
                return next == ManualRollbackProfileStatus::Revoked;
            }
            ManualRollbackProfileStatus::Revoked => {
                match next {
                    ManualRollbackProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualRollbackProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rollback profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> ManualRollbackProfile {
        ManualRollbackProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            visible: self.visible,
            discount: self.discount,
            archived: self.archived,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a manual rollback profile in status requested.
pub fn new_manual_rollback_profile(code: &str, title: &str) -> ManualRollbackProfile {
    ManualRollbackProfile {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        visible: false,
        discount: 0.0,
        archived: false,
        audience: "".to_string(),
        status: ManualRollbackProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_rollback_profile("code-225", "title-986");
        assert_eq!(item.status, ManualRollbackProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_rollback_profile("code-225", "title-986");
        if item.move_to(ManualRollbackProfileStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRollbackProfileStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_rollback_profile("code-225", "title-986");
        item.status = ManualRollbackProfileStatus::Expired;
        assert!(
            item.move_to(ManualRollbackProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
