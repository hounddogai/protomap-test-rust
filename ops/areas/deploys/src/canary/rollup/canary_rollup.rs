use super::*;

use acme_kit::text;

/// Where a canary rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanaryRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanaryRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRollup {
    pub code: String,
    pub title: String,
    pub label: String,
    pub batch_size: i64,
    pub ends_at: i64,
    pub name: String,
    pub active: bool,
    pub retries: i64,
    pub rotation_days: i64,
    pub status: CanaryRollupStatus,
}

impl CanaryRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryRollupStatus) -> bool {
        match self.status {
            CanaryRollupStatus::Proposed => {
                match next {
                    CanaryRollupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryRollupStatus::Approved => {
                match next {
                    CanaryRollupStatus::Rejected | CanaryRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryRollupStatus::Rejected => {
                return next == CanaryRollupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary rollup {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.rotation_days as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryRollup {
        CanaryRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            ends_at: self.ends_at,
            name: self.name.clone(),
            active: self.active,
            retries: self.retries,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a canary rollup in status proposed.
pub fn new_canary_rollup(code: &str, title: &str) -> CanaryRollup {
    CanaryRollup {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        batch_size: 0,
        ends_at: 0,
        name: "".to_string(),
        active: false,
        retries: 0,
        rotation_days: 0,
        status: CanaryRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_canary_rollup("code-326", "title-531");
        assert_eq!(item.status, CanaryRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_canary_rollup("code-326", "title-531");
        if item.move_to(CanaryRollupStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryRollupStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_canary_rollup("code-326", "title-531");
        item.status = CanaryRollupStatus::Withdrawn;
        assert!(
            item.move_to(CanaryRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
