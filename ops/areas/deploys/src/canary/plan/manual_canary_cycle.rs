use super::*;

use acme_kit::text;

/// Where a manual canary cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualCanaryCycleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualCanaryCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualCanaryCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual canary cycle as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanaryCycle {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub pinned: bool,
    pub grace_days: i64,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub burst: i64,
    pub owner_id: String,
    pub rotation_days: i64,
    pub active: bool,
    pub status: ManualCanaryCycleStatus,
}

impl ManualCanaryCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualCanaryCycleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualCanaryCycleStatus) -> bool {
        match self.status {
            ManualCanaryCycleStatus::Open => {
                return next == ManualCanaryCycleStatus::Reopened;
            }
            ManualCanaryCycleStatus::InReview => {
                match next {
                    ManualCanaryCycleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCanaryCycleStatus::Resolved => {
                return next == ManualCanaryCycleStatus::Archived;
            }
            ManualCanaryCycleStatus::Reopened => {
                match next {
                    ManualCanaryCycleStatus::Archived => {
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

    pub fn move_to(&mut self, next: ManualCanaryCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual canary cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual canary cycle", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quota_units as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualCanaryCycle {
        ManualCanaryCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            grace_days: self.grace_days,
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            burst: self.burst,
            owner_id: self.owner_id.clone(),
            rotation_days: self.rotation_days,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual canary cycle in status open.
pub fn new_manual_canary_cycle(code: &str, name: &str) -> ManualCanaryCycle {
    ManualCanaryCycle {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        pinned: false,
        grace_days: 0,
        quota_units: 0,
        tags: Vec::new(),
        burst: 0,
        owner_id: "".to_string(),
        rotation_days: 0,
        active: false,
        status: ManualCanaryCycleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_canary_cycle("code-68", "name-391");
        assert_eq!(item.status, ManualCanaryCycleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_manual_canary_cycle("code-68", "name-391");
        if item.move_to(ManualCanaryCycleStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualCanaryCycleStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_canary_cycle("code-68", "name-391");
        item.status = ManualCanaryCycleStatus::Archived;
        assert!(
            item.move_to(ManualCanaryCycleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
