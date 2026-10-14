use super::*;

use acme_kit::text;

/// Where a manual tenant notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTenantNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualTenantNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ManualTenantNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual tenant notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantNotice {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub archived: bool,
    pub batch_size: i64,
    pub external_ref: String,
    pub status: ManualTenantNoticeStatus,
}

impl ManualTenantNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTenantNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTenantNoticeStatus) -> bool {
        match self.status {
            ManualTenantNoticeStatus::Planned => {
                return next == ManualTenantNoticeStatus::Cancelled;
            }
            ManualTenantNoticeStatus::Scheduled => {
                match next {
                    ManualTenantNoticeStatus::InProgress | ManualTenantNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTenantNoticeStatus::InProgress => {
                match next {
                    ManualTenantNoticeStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTenantNoticeStatus::Completed => {
                match next {
                    ManualTenantNoticeStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ManualTenantNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual tenant notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual tenant notice", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.batch_size as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTenantNotice {
        ManualTenantNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            archived: self.archived,
            batch_size: self.batch_size,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual tenant notice in status planned.
pub fn new_manual_tenant_notice(code: &str, name: &str) -> ManualTenantNotice {
    ManualTenantNotice {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        archived: false,
        batch_size: 0,
        external_ref: "".to_string(),
        status: ManualTenantNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_tenant_notice("code-899", "name-170");
        assert_eq!(item.status, ManualTenantNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_manual_tenant_notice("code-899", "name-170");
        if item.move_to(ManualTenantNoticeStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTenantNoticeStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_tenant_notice("code-899", "name-170");
        item.status = ManualTenantNoticeStatus::Cancelled;
        assert!(
            item.move_to(ManualTenantNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
