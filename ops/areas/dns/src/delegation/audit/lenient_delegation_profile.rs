use super::*;

use acme_kit::text;

/// Where a lenient delegation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDelegationProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientDelegationProfileStatus {
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

impl std::fmt::Display for LenientDelegationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient delegation profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationProfile {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub priority: i64,
    pub attempts: i64,
    pub ceiling: f64,
    pub fingerprint: String,
    pub budget: f64,
    pub approved: bool,
    pub principal: String,
    pub status: LenientDelegationProfileStatus,
}

impl LenientDelegationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDelegationProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDelegationProfileStatus) -> bool {
        match self.status {
            LenientDelegationProfileStatus::Draft => {
                return next == LenientDelegationProfileStatus::Pending
                    || next == LenientDelegationProfileStatus::Suspended;
            }
            LenientDelegationProfileStatus::Pending => {
                return next == LenientDelegationProfileStatus::Suspended;
            }
            LenientDelegationProfileStatus::Active => {
                match next {
                    LenientDelegationProfileStatus::Suspended | LenientDelegationProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDelegationProfileStatus::Suspended => {
                return next == LenientDelegationProfileStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDelegationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient delegation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.attempts as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDelegationProfile {
        LenientDelegationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            priority: self.priority,
            attempts: self.attempts,
            ceiling: self.ceiling,
            fingerprint: self.fingerprint.clone(),
            budget: self.budget,
            approved: self.approved,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a lenient delegation profile in status draft.
pub fn new_lenient_delegation_profile(code: &str, title: &str) -> LenientDelegationProfile {
    LenientDelegationProfile {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        priority: 0,
        attempts: 0,
        ceiling: 0.0,
        fingerprint: "".to_string(),
        budget: 0.0,
        approved: false,
        principal: "".to_string(),
        status: LenientDelegationProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_delegation_profile("code-857", "title-375");
        assert_eq!(item.status, LenientDelegationProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_delegation_profile("code-857", "title-375");
        if item.move_to(LenientDelegationProfileStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDelegationProfileStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_delegation_profile("code-857", "title-375");
        item.status = LenientDelegationProfileStatus::Closed;
        assert!(
            item.move_to(LenientDelegationProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
