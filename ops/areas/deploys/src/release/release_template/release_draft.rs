use super::*;

use acme_kit::text;

/// Where a release draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReleaseDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReleaseDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDraft {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub host: String,
    pub rotation_days: i64,
    pub notes: String,
    pub timezone: String,
    pub region: String,
    pub unit: String,
    pub batch_size: i64,
    pub status: ReleaseDraftStatus,
}

impl ReleaseDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseDraftStatus) -> bool {
        match self.status {
            ReleaseDraftStatus::Created => {
                match next {
                    ReleaseDraftStatus::Validated | ReleaseDraftStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseDraftStatus::Validated => {
                return next == ReleaseDraftStatus::Published || next == ReleaseDraftStatus::Retired;
            }
            ReleaseDraftStatus::Published => {
                return next == ReleaseDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.batch_size as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseDraft {
        ReleaseDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            host: self.host.clone(),
            rotation_days: self.rotation_days,
            notes: self.notes.clone(),
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            unit: self.unit.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a release draft in status created.
pub fn new_release_draft(code: &str, name: &str) -> ReleaseDraft {
    ReleaseDraft {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        host: "".to_string(),
        rotation_days: 0,
        notes: "".to_string(),
        timezone: "".to_string(),
        region: "".to_string(),
        unit: "".to_string(),
        batch_size: 0,
        status: ReleaseDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_release_draft("code-569", "name-383");
        assert_eq!(item.status, ReleaseDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_release_draft("code-569", "name-383");
        if item.move_to(ReleaseDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_release_draft("code-569", "name-383");
        item.status = ReleaseDraftStatus::Retired;
        assert!(
            item.move_to(ReleaseDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
