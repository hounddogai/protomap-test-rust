use super::*;

use acme_kit::text;

/// Where a scheduled tenant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTenantAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledTenantAgreementStatus {
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

impl std::fmt::Display for ScheduledTenantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled tenant agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantAgreement {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub name: String,
    pub archived: bool,
    pub score: f64,
    pub port: i64,
    pub locale: String,
    pub contact_email: String,
    pub principal: String,
    pub channel: String,
    pub status: ScheduledTenantAgreementStatus,
}

impl ScheduledTenantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTenantAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTenantAgreementStatus) -> bool {
        match self.status {
            ScheduledTenantAgreementStatus::Idle => {
                match next {
                    ScheduledTenantAgreementStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantAgreementStatus::Warming => {
                match next {
                    ScheduledTenantAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantAgreementStatus::Ready => {
                return next == ScheduledTenantAgreementStatus::Draining
                    || next == ScheduledTenantAgreementStatus::Stopped;
            }
            ScheduledTenantAgreementStatus::Draining => {
                return next == ScheduledTenantAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTenantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled tenant agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.port as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTenantAgreement {
        ScheduledTenantAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            name: self.name.clone(),
            archived: self.archived,
            score: self.score,
            port: self.port,
            locale: self.locale.clone(),
            contact_email: self.contact_email.clone(),
            principal: self.principal.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled tenant agreement in status idle.
pub fn new_scheduled_tenant_agreement(code: &str, label: &str) -> ScheduledTenantAgreement {
    ScheduledTenantAgreement {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        name: "".to_string(),
        archived: false,
        score: 0.0,
        port: 0,
        locale: "".to_string(),
        contact_email: "".to_string(),
        principal: "".to_string(),
        channel: "".to_string(),
        status: ScheduledTenantAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_tenant_agreement("code-383", "label-116");
        assert_eq!(item.status, ScheduledTenantAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_tenant_agreement("code-383", "label-116");
        if item.move_to(ScheduledTenantAgreementStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTenantAgreementStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_tenant_agreement("code-383", "label-116");
        item.status = ScheduledTenantAgreementStatus::Stopped;
        assert!(
            item.move_to(ScheduledTenantAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
