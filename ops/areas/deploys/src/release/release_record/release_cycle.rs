use super::*;

use acme_kit::text;

/// Where a release cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReleaseCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReleaseCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseCycle {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub burst: i64,
    pub rotation_days: i64,
    pub margin: f64,
    pub status: ReleaseCycleStatus,
}

impl ReleaseCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseCycleStatus) -> bool {
        match self.status {
            ReleaseCycleStatus::Proposed => {
                match next {
                    ReleaseCycleStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseCycleStatus::Approved => {
                return next == ReleaseCycleStatus::Rejected;
            }
            ReleaseCycleStatus::Rejected => {
                match next {
                    ReleaseCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ReleaseCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release cycle {} cannot move from {} to {}",
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

    pub fn net_value(&self) -> f64 {
        ((self.rotation_days as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseCycle {
        ReleaseCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            burst: self.burst,
            rotation_days: self.rotation_days,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a release cycle in status proposed.
pub fn new_release_cycle(code: &str, title: &str) -> ReleaseCycle {
    ReleaseCycle {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        burst: 0,
        rotation_days: 0,
        margin: 0.0,
        status: ReleaseCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_release_cycle("code-240", "title-793");
        assert_eq!(item.status, ReleaseCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_release_cycle("code-240", "title-793");
        if item.move_to(ReleaseCycleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseCycleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_release_cycle("code-240", "title-793");
        item.status = ReleaseCycleStatus::Withdrawn;
        assert!(
            item.move_to(ReleaseCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
