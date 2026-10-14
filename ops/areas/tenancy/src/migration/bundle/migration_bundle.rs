use super::*;

use acme_kit::text;

/// Where a migration bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MigrationBundleStatus {
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

impl std::fmt::Display for MigrationBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBundle {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub starts_at: i64,
    pub timezone: String,
    pub quota_units: i64,
    pub channel: String,
    pub tags: Vec<String>,
    pub status: MigrationBundleStatus,
}

impl MigrationBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationBundleStatus) -> bool {
        match self.status {
            MigrationBundleStatus::Idle => {
                return next == MigrationBundleStatus::Draining;
            }
            MigrationBundleStatus::Warming => {
                match next {
                    MigrationBundleStatus::Ready | MigrationBundleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationBundleStatus::Ready => {
                return next == MigrationBundleStatus::Draining;
            }
            MigrationBundleStatus::Draining => {
                match next {
                    MigrationBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MigrationBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quota_units as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationBundle {
        MigrationBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            starts_at: self.starts_at,
            timezone: self.timezone.clone(),
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a migration bundle in status idle.
pub fn new_migration_bundle(code: &str, label: &str) -> MigrationBundle {
    MigrationBundle {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        starts_at: 0,
        timezone: "".to_string(),
        quota_units: 0,
        channel: "".to_string(),
        tags: Vec::new(),
        status: MigrationBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_migration_bundle("code-612", "label-265");
        assert_eq!(item.status, MigrationBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_migration_bundle("code-612", "label-265");
        if item.move_to(MigrationBundleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationBundleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_migration_bundle("code-612", "label-265");
        item.status = MigrationBundleStatus::Stopped;
        assert!(
            item.move_to(MigrationBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
