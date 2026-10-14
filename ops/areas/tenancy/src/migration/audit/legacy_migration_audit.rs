use super::*;

use acme_kit::text;

/// Where a legacy migration audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMigrationAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyMigrationAuditStatus {
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

impl std::fmt::Display for LegacyMigrationAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy migration audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAudit {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub ends_at: i64,
    pub burst: i64,
    pub status: LegacyMigrationAuditStatus,
}

impl LegacyMigrationAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMigrationAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMigrationAuditStatus) -> bool {
        match self.status {
            LegacyMigrationAuditStatus::Idle => {
                return next == LegacyMigrationAuditStatus::Ready || next == LegacyMigrationAuditStatus::Stopped;
            }
            LegacyMigrationAuditStatus::Warming => {
                return next == LegacyMigrationAuditStatus::Stopped;
            }
            LegacyMigrationAuditStatus::Ready => {
                return next == LegacyMigrationAuditStatus::Stopped;
            }
            LegacyMigrationAuditStatus::Draining => {
                return next == LegacyMigrationAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyMigrationAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy migration audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy migration audit", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.burst as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyMigrationAudit {
        LegacyMigrationAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            ends_at: self.ends_at,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy migration audit in status idle.
pub fn new_legacy_migration_audit(code: &str, label: &str) -> LegacyMigrationAudit {
    LegacyMigrationAudit {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        ends_at: 0,
        burst: 0,
        status: LegacyMigrationAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_migration_audit("code-112", "label-276");
        assert_eq!(item.status, LegacyMigrationAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_legacy_migration_audit("code-112", "label-276");
        if item.move_to(LegacyMigrationAuditStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyMigrationAuditStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_migration_audit("code-112", "label-276");
        item.status = LegacyMigrationAuditStatus::Stopped;
        assert!(
            item.move_to(LegacyMigrationAuditStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
