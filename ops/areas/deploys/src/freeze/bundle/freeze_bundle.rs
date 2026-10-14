use super::*;

use acme_kit::text;

/// Where a freeze bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FreezeBundleStatus {
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

impl std::fmt::Display for FreezeBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeBundle {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub floor_value: f64,
    pub revision: i64,
    pub created_at: i64,
    pub status: FreezeBundleStatus,
}

impl FreezeBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeBundleStatus) -> bool {
        match self.status {
            FreezeBundleStatus::Planned => {
                match next {
                    FreezeBundleStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeBundleStatus::Scheduled => {
                match next {
                    FreezeBundleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeBundleStatus::InProgress => {
                return next == FreezeBundleStatus::Completed;
            }
            FreezeBundleStatus::Completed => {
                return next == FreezeBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.created_at as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeBundle {
        FreezeBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            floor_value: self.floor_value,
            revision: self.revision,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze bundle in status planned.
pub fn new_freeze_bundle(code: &str, title: &str) -> FreezeBundle {
    FreezeBundle {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        floor_value: 0.0,
        revision: 0,
        created_at: 0,
        status: FreezeBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_freeze_bundle("code-54", "title-324");
        assert_eq!(item.status, FreezeBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_freeze_bundle("code-54", "title-324");
        if item.move_to(FreezeBundleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeBundleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_freeze_bundle("code-54", "title-324");
        item.status = FreezeBundleStatus::Cancelled;
        assert!(
            item.move_to(FreezeBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
