use super::*;

use acme_kit::text;

/// Where a primary release manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryReleaseManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryReleaseManifestStatus {
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

impl std::fmt::Display for PrimaryReleaseManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary release manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseManifest {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub priority: i64,
    pub ceiling: f64,
    pub burst: i64,
    pub country: String,
    pub fingerprint: String,
    pub approved: bool,
    pub maximum: f64,
    pub status: PrimaryReleaseManifestStatus,
}

impl PrimaryReleaseManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryReleaseManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryReleaseManifestStatus) -> bool {
        match self.status {
            PrimaryReleaseManifestStatus::Planned => {
                match next {
                    PrimaryReleaseManifestStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryReleaseManifestStatus::Scheduled => {
                return next == PrimaryReleaseManifestStatus::Completed
                    || next == PrimaryReleaseManifestStatus::Cancelled;
            }
            PrimaryReleaseManifestStatus::InProgress => {
                return next == PrimaryReleaseManifestStatus::Completed
                    || next == PrimaryReleaseManifestStatus::Cancelled;
            }
            PrimaryReleaseManifestStatus::Completed => {
                return next == PrimaryReleaseManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryReleaseManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary release manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary release manifest", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryReleaseManifest {
        PrimaryReleaseManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            priority: self.priority,
            ceiling: self.ceiling,
            burst: self.burst,
            country: self.country.clone(),
            fingerprint: self.fingerprint.clone(),
            approved: self.approved,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary release manifest in status planned.
pub fn new_primary_release_manifest(code: &str, title: &str) -> PrimaryReleaseManifest {
    PrimaryReleaseManifest {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        priority: 0,
        ceiling: 0.0,
        burst: 0,
        country: "".to_string(),
        fingerprint: "".to_string(),
        approved: false,
        maximum: 0.0,
        status: PrimaryReleaseManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_release_manifest("code-262", "title-988");
        assert_eq!(item.status, PrimaryReleaseManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_release_manifest("code-262", "title-988");
        if item.move_to(PrimaryReleaseManifestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryReleaseManifestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_release_manifest("code-262", "title-988");
        item.status = PrimaryReleaseManifestStatus::Cancelled;
        assert!(
            item.move_to(PrimaryReleaseManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
