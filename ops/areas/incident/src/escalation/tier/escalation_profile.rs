use super::*;

use acme_kit::text;

/// Where a escalation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl EscalationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for EscalationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation profile as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationProfile {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub tenant_id: String,
    pub grace_days: i64,
    pub status: EscalationProfileStatus,
}

impl EscalationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationProfileStatus) -> bool {
        match self.status {
            EscalationProfileStatus::Requested => {
                match next {
                    EscalationProfileStatus::Revoked | EscalationProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationProfileStatus::Granted => {
                match next {
                    EscalationProfileStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationProfileStatus::Revoked => {
                return next == EscalationProfileStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> EscalationProfile {
        EscalationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a escalation profile in status requested.
pub fn new_escalation_profile(code: &str, title: &str) -> EscalationProfile {
    EscalationProfile {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        tenant_id: "".to_string(),
        grace_days: 0,
        status: EscalationProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_escalation_profile("code-895", "title-184");
        assert_eq!(item.status, EscalationProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_escalation_profile("code-895", "title-184");
        if item.move_to(EscalationProfileStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationProfileStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_escalation_profile("code-895", "title-184");
        item.status = EscalationProfileStatus::Expired;
        assert!(
            item.move_to(EscalationProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
