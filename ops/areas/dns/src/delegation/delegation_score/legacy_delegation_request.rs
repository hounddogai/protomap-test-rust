use super::*;

use acme_kit::text;

/// Where a legacy delegation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDelegationRequestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyDelegationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for LegacyDelegationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy delegation request as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDelegationRequest {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub approved: bool,
    pub external_ref: String,
    pub capacity: i64,
    pub contact_email: String,
    pub reviewer: String,
    pub duration_minutes: i64,
    pub quantity: i64,
    pub category: String,
    pub status: LegacyDelegationRequestStatus,
}

impl LegacyDelegationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDelegationRequestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDelegationRequestStatus) -> bool {
        match self.status {
            LegacyDelegationRequestStatus::Idle => {
                return next == LegacyDelegationRequestStatus::Ready || next == LegacyDelegationRequestStatus::Draining;
            }
            LegacyDelegationRequestStatus::Warming => {
                return next == LegacyDelegationRequestStatus::Stopped;
            }
            LegacyDelegationRequestStatus::Ready => {
                match next {
                    LegacyDelegationRequestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDelegationRequestStatus::Draining => {
                match next {
                    LegacyDelegationRequestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LegacyDelegationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy delegation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy delegation request", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDelegationRequest {
        LegacyDelegationRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            approved: self.approved,
            external_ref: self.external_ref.clone(),
            capacity: self.capacity,
            contact_email: self.contact_email.clone(),
            reviewer: self.reviewer.clone(),
            duration_minutes: self.duration_minutes,
            quantity: self.quantity,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a legacy delegation request in status idle.
pub fn new_legacy_delegation_request(code: &str, title: &str) -> LegacyDelegationRequest {
    LegacyDelegationRequest {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        approved: false,
        external_ref: "".to_string(),
        capacity: 0,
        contact_email: "".to_string(),
        reviewer: "".to_string(),
        duration_minutes: 0,
        quantity: 0,
        category: "".to_string(),
        status: LegacyDelegationRequestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_delegation_request("code-81", "title-895");
        assert_eq!(item.status, LegacyDelegationRequestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_legacy_delegation_request("code-81", "title-895");
        if item.move_to(LegacyDelegationRequestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDelegationRequestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_delegation_request("code-81", "title-895");
        item.status = LegacyDelegationRequestStatus::Stopped;
        assert!(
            item.move_to(LegacyDelegationRequestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
