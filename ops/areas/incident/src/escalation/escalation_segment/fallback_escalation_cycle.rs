use super::*;

use acme_kit::text;

/// Where a fallback escalation cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEscalationCycleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackEscalationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackEscalationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback escalation cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationCycle {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub threshold: f64,
    pub external_ref: String,
    pub weight: f64,
    pub tenant_id: String,
    pub rotation_days: i64,
    pub title: String,
    pub contact_email: String,
    pub grace_days: i64,
    pub status: FallbackEscalationCycleStatus,
}

impl FallbackEscalationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEscalationCycleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEscalationCycleStatus) -> bool {
        match self.status {
            FallbackEscalationCycleStatus::Requested => {
                match next {
                    FallbackEscalationCycleStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEscalationCycleStatus::Granted => {
                match next {
                    FallbackEscalationCycleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEscalationCycleStatus::Revoked => {
                match next {
                    FallbackEscalationCycleStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackEscalationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback escalation cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.weight * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackEscalationCycle {
        FallbackEscalationCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            threshold: self.threshold,
            external_ref: self.external_ref.clone(),
            weight: self.weight,
            tenant_id: self.tenant_id.clone(),
            rotation_days: self.rotation_days,
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback escalation cycle in status requested.
pub fn new_fallback_escalation_cycle(code: &str, label: &str) -> FallbackEscalationCycle {
    FallbackEscalationCycle {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        threshold: 0.0,
        external_ref: "".to_string(),
        weight: 0.0,
        tenant_id: "".to_string(),
        rotation_days: 0,
        title: "".to_string(),
        contact_email: "".to_string(),
        grace_days: 0,
        status: FallbackEscalationCycleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_escalation_cycle("code-172", "label-122");
        assert_eq!(item.status, FallbackEscalationCycleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_fallback_escalation_cycle("code-172", "label-122");
        if item.move_to(FallbackEscalationCycleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackEscalationCycleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_escalation_cycle("code-172", "label-122");
        item.status = FallbackEscalationCycleStatus::Expired;
        assert!(
            item.move_to(FallbackEscalationCycleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
