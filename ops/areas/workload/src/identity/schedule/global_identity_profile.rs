use super::*;

use acme_kit::text;

/// Where a global identity profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIdentityProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalIdentityProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for GlobalIdentityProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global identity profile as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityProfile {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub reviewer: String,
    pub margin: f64,
    pub key_id: String,
    pub status: GlobalIdentityProfileStatus,
}

impl GlobalIdentityProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIdentityProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIdentityProfileStatus) -> bool {
        match self.status {
            GlobalIdentityProfileStatus::Draft => {
                return next == GlobalIdentityProfileStatus::Pending || next == GlobalIdentityProfileStatus::Active;
            }
            GlobalIdentityProfileStatus::Pending => {
                match next {
                    GlobalIdentityProfileStatus::Active | GlobalIdentityProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityProfileStatus::Active => {
                match next {
                    GlobalIdentityProfileStatus::Suspended | GlobalIdentityProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityProfileStatus::Suspended => {
                match next {
                    GlobalIdentityProfileStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalIdentityProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global identity profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        (self.margin * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalIdentityProfile {
        GlobalIdentityProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            margin: self.margin,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global identity profile in status draft.
pub fn new_global_identity_profile(code: &str, name: &str) -> GlobalIdentityProfile {
    GlobalIdentityProfile {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        reviewer: "".to_string(),
        margin: 0.0,
        key_id: "".to_string(),
        status: GlobalIdentityProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_identity_profile("code-168", "name-630");
        assert_eq!(item.status, GlobalIdentityProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_global_identity_profile("code-168", "name-630");
        if item.move_to(GlobalIdentityProfileStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalIdentityProfileStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_identity_profile("code-168", "name-630");
        item.status = GlobalIdentityProfileStatus::Closed;
        assert!(
            item.move_to(GlobalIdentityProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
