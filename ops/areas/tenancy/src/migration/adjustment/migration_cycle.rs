use super::*;

use acme_kit::text;

/// Where a migration cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCycle {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub window_days: i64,
    pub budget: f64,
    pub channel: String,
    pub discount: f64,
    pub pinned: bool,
    pub host: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub status: MigrationCycleStatus,
}

impl MigrationCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationCycleStatus) -> bool {
        match self.status {
            MigrationCycleStatus::Proposed => {
                return next == MigrationCycleStatus::Approved;
            }
            MigrationCycleStatus::Approved => {
                return next == MigrationCycleStatus::Rejected || next == MigrationCycleStatus::Withdrawn;
            }
            MigrationCycleStatus::Rejected => {
                match next {
                    MigrationCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MigrationCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.window_days as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationCycle {
        MigrationCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            window_days: self.window_days,
            budget: self.budget,
            channel: self.channel.clone(),
            discount: self.discount,
            pinned: self.pinned,
            host: self.host.clone(),
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a migration cycle in status proposed.
pub fn new_migration_cycle(code: &str, label: &str) -> MigrationCycle {
    MigrationCycle {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        window_days: 0,
        budget: 0.0,
        channel: "".to_string(),
        discount: 0.0,
        pinned: false,
        host: "".to_string(),
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        status: MigrationCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_migration_cycle("code-383", "label-103");
        assert_eq!(item.status, MigrationCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_migration_cycle("code-383", "label-103");
        if item.move_to(MigrationCycleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationCycleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_migration_cycle("code-383", "label-103");
        item.status = MigrationCycleStatus::Withdrawn;
        assert!(
            item.move_to(MigrationCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
