use super::*;

use acme_kit::text;

/// Where a migration window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MigrationWindowStatus {
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

impl std::fmt::Display for MigrationWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationWindow {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub country: String,
    pub owner_id: String,
    pub retries: i64,
    pub key_id: String,
    pub tags: Vec<String>,
    pub contact_email: String,
    pub status: MigrationWindowStatus,
}

impl MigrationWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationWindowStatus) -> bool {
        match self.status {
            MigrationWindowStatus::Idle => {
                match next {
                    MigrationWindowStatus::Ready | MigrationWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationWindowStatus::Warming => {
                return next == MigrationWindowStatus::Draining;
            }
            MigrationWindowStatus::Ready => {
                match next {
                    MigrationWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationWindowStatus::Draining => {
                return next == MigrationWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_total(&self) -> f64 {
        (self.discount * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationWindow {
        MigrationWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            country: self.country.clone(),
            owner_id: self.owner_id.clone(),
            retries: self.retries,
            key_id: self.key_id.clone(),
            tags: self.tags.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a migration window in status idle.
pub fn new_migration_window(code: &str, label: &str) -> MigrationWindow {
    MigrationWindow {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        country: "".to_string(),
        owner_id: "".to_string(),
        retries: 0,
        key_id: "".to_string(),
        tags: Vec::new(),
        contact_email: "".to_string(),
        status: MigrationWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_migration_window("code-280", "label-454");
        assert_eq!(item.status, MigrationWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_migration_window("code-280", "label-454");
        if item.move_to(MigrationWindowStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationWindowStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_migration_window("code-280", "label-454");
        item.status = MigrationWindowStatus::Stopped;
        assert!(
            item.move_to(MigrationWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
