use super::*;

use acme_kit::text;

/// Where a plan snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PlanSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PlanSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSnapshot {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub active: bool,
    pub timezone: String,
    pub maximum: f64,
    pub duration_minutes: i64,
    pub status: PlanSnapshotStatus,
}

impl PlanSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanSnapshotStatus) -> bool {
        match self.status {
            PlanSnapshotStatus::Created => {
                return next == PlanSnapshotStatus::Published;
            }
            PlanSnapshotStatus::Validated => {
                return next == PlanSnapshotStatus::Retired;
            }
            PlanSnapshotStatus::Published => {
                match next {
                    PlanSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: PlanSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "plan snapshot", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.duration_minutes as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanSnapshot {
        PlanSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            active: self.active,
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a plan snapshot in status created.
pub fn new_plan_snapshot(code: &str, label: &str) -> PlanSnapshot {
    PlanSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        active: false,
        timezone: "".to_string(),
        maximum: 0.0,
        duration_minutes: 0,
        status: PlanSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_plan_snapshot("code-609", "label-750");
        assert_eq!(item.status, PlanSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_plan_snapshot("code-609", "label-750");
        if item.move_to(PlanSnapshotStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanSnapshotStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_plan_snapshot("code-609", "label-750");
        item.status = PlanSnapshotStatus::Retired;
        assert!(
            item.move_to(PlanSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
