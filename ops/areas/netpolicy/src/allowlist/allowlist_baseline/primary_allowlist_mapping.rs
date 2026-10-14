use super::*;

use acme_kit::text;

/// Where a primary allowlist mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryAllowlistMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryAllowlistMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PrimaryAllowlistMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary allowlist mapping as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMapping {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub owner_id: String,
    pub weight: f64,
    pub status: PrimaryAllowlistMappingStatus,
}

impl PrimaryAllowlistMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryAllowlistMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryAllowlistMappingStatus) -> bool {
        match self.status {
            PrimaryAllowlistMappingStatus::New => {
                return next == PrimaryAllowlistMappingStatus::Running;
            }
            PrimaryAllowlistMappingStatus::Queued => {
                match next {
                    PrimaryAllowlistMappingStatus::Running | PrimaryAllowlistMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryAllowlistMappingStatus::Running => {
                match next {
                    PrimaryAllowlistMappingStatus::Done | PrimaryAllowlistMappingStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryAllowlistMappingStatus::Done => {
                return next == PrimaryAllowlistMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryAllowlistMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary allowlist mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryAllowlistMapping {
        PrimaryAllowlistMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary allowlist mapping in status new.
pub fn new_primary_allowlist_mapping(code: &str, title: &str) -> PrimaryAllowlistMapping {
    PrimaryAllowlistMapping {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        owner_id: "".to_string(),
        weight: 0.0,
        status: PrimaryAllowlistMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_allowlist_mapping("code-625", "title-636");
        assert_eq!(item.status, PrimaryAllowlistMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_allowlist_mapping("code-625", "title-636");
        if item.move_to(PrimaryAllowlistMappingStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryAllowlistMappingStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_allowlist_mapping("code-625", "title-636");
        item.status = PrimaryAllowlistMappingStatus::Failed;
        assert!(
            item.move_to(PrimaryAllowlistMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
