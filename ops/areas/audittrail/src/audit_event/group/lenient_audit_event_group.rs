use super::*;

use acme_kit::text;

/// Where a lenient audit event group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAuditEventGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientAuditEventGroupStatus {
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

impl std::fmt::Display for LenientAuditEventGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient audit event group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventGroup {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub timezone: String,
    pub region: String,
    pub status: LenientAuditEventGroupStatus,
}

impl LenientAuditEventGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAuditEventGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAuditEventGroupStatus) -> bool {
        match self.status {
            LenientAuditEventGroupStatus::Idle => {
                match next {
                    LenientAuditEventGroupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAuditEventGroupStatus::Warming => {
                return next == LenientAuditEventGroupStatus::Draining;
            }
            LenientAuditEventGroupStatus::Ready => {
                return next == LenientAuditEventGroupStatus::Draining;
            }
            LenientAuditEventGroupStatus::Draining => {
                return next == LenientAuditEventGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientAuditEventGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient audit event group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient audit event group", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> LenientAuditEventGroup {
        LenientAuditEventGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient audit event group in status idle.
pub fn new_lenient_audit_event_group(code: &str, title: &str) -> LenientAuditEventGroup {
    LenientAuditEventGroup {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        timezone: "".to_string(),
        region: "".to_string(),
        status: LenientAuditEventGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_audit_event_group("code-915", "title-844");
        assert_eq!(item.status, LenientAuditEventGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_lenient_audit_event_group("code-915", "title-844");
        if item.move_to(LenientAuditEventGroupStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientAuditEventGroupStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_audit_event_group("code-915", "title-844");
        item.status = LenientAuditEventGroupStatus::Stopped;
        assert!(
            item.move_to(LenientAuditEventGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
