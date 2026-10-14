use super::*;

use acme_kit::text;

/// Where a legacy tenant notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTenantNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyTenantNoticeStatus {
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

impl std::fmt::Display for LegacyTenantNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy tenant notice as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantNotice {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub pinned: bool,
    pub rate: f64,
    pub window_days: i64,
    pub duration_minutes: i64,
    pub port: i64,
    pub cost: f64,
    pub status: LegacyTenantNoticeStatus,
}

impl LegacyTenantNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTenantNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTenantNoticeStatus) -> bool {
        match self.status {
            LegacyTenantNoticeStatus::Planned => {
                return next == LegacyTenantNoticeStatus::Scheduled || next == LegacyTenantNoticeStatus::Completed;
            }
            LegacyTenantNoticeStatus::Scheduled => {
                match next {
                    LegacyTenantNoticeStatus::InProgress | LegacyTenantNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTenantNoticeStatus::InProgress => {
                return next == LegacyTenantNoticeStatus::Completed || next == LegacyTenantNoticeStatus::Cancelled;
            }
            LegacyTenantNoticeStatus::Completed => {
                return next == LegacyTenantNoticeStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyTenantNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy tenant notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy tenant notice", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.version as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTenantNotice {
        LegacyTenantNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            pinned: self.pinned,
            rate: self.rate,
            window_days: self.window_days,
            duration_minutes: self.duration_minutes,
            port: self.port,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy tenant notice in status planned.
pub fn new_legacy_tenant_notice(code: &str, title: &str) -> LegacyTenantNotice {
    LegacyTenantNotice {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        pinned: false,
        rate: 0.0,
        window_days: 0,
        duration_minutes: 0,
        port: 0,
        cost: 0.0,
        status: LegacyTenantNoticeStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_tenant_notice("code-827", "title-30");
        assert_eq!(item.status, LegacyTenantNoticeStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_legacy_tenant_notice("code-827", "title-30");
        if item.move_to(LegacyTenantNoticeStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTenantNoticeStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_tenant_notice("code-827", "title-30");
        item.status = LegacyTenantNoticeStatus::Cancelled;
        assert!(
            item.move_to(LegacyTenantNoticeStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
