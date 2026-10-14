use super::*;

use acme_kit::text;

/// Where a primary rollout rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRolloutRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryRolloutRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PrimaryRolloutRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rollout rollup as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutRollup {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub created_at: i64,
    pub rate: f64,
    pub reviewer: String,
    pub rotation_days: i64,
    pub unit: String,
    pub key_id: String,
    pub starts_at: i64,
    pub expires_at: i64,
    pub status: PrimaryRolloutRollupStatus,
}

impl PrimaryRolloutRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRolloutRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRolloutRollupStatus) -> bool {
        match self.status {
            PrimaryRolloutRollupStatus::Draft => {
                return next == PrimaryRolloutRollupStatus::Suspended || next == PrimaryRolloutRollupStatus::Closed;
            }
            PrimaryRolloutRollupStatus::Pending => {
                return next == PrimaryRolloutRollupStatus::Suspended;
            }
            PrimaryRolloutRollupStatus::Active => {
                return next == PrimaryRolloutRollupStatus::Closed;
            }
            PrimaryRolloutRollupStatus::Suspended => {
                match next {
                    PrimaryRolloutRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryRolloutRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rollout rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.rotation_days as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRolloutRollup {
        PrimaryRolloutRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            created_at: self.created_at,
            rate: self.rate,
            reviewer: self.reviewer.clone(),
            rotation_days: self.rotation_days,
            unit: self.unit.clone(),
            key_id: self.key_id.clone(),
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary rollout rollup in status draft.
pub fn new_primary_rollout_rollup(code: &str, title: &str) -> PrimaryRolloutRollup {
    PrimaryRolloutRollup {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        created_at: 0,
        rate: 0.0,
        reviewer: "".to_string(),
        rotation_days: 0,
        unit: "".to_string(),
        key_id: "".to_string(),
        starts_at: 0,
        expires_at: 0,
        status: PrimaryRolloutRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_rollout_rollup("code-86", "title-826");
        assert_eq!(item.status, PrimaryRolloutRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_primary_rollout_rollup("code-86", "title-826");
        if item.move_to(PrimaryRolloutRollupStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRolloutRollupStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_rollout_rollup("code-86", "title-826");
        item.status = PrimaryRolloutRollupStatus::Closed;
        assert!(
            item.move_to(PrimaryRolloutRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
