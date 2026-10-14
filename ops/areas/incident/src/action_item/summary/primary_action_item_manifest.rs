use super::*;

use acme_kit::text;

/// Where a primary action item manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryActionItemManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryActionItemManifestStatus {
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

impl std::fmt::Display for PrimaryActionItemManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary action item manifest as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifest {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub active: bool,
    pub duration_minutes: i64,
    pub cluster: String,
    pub pinned: bool,
    pub status: PrimaryActionItemManifestStatus,
}

impl PrimaryActionItemManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryActionItemManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryActionItemManifestStatus) -> bool {
        match self.status {
            PrimaryActionItemManifestStatus::Open => {
                return next == PrimaryActionItemManifestStatus::Reopened
                    || next == PrimaryActionItemManifestStatus::Archived;
            }
            PrimaryActionItemManifestStatus::InReview => {
                match next {
                    PrimaryActionItemManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryActionItemManifestStatus::Resolved => {
                match next {
                    PrimaryActionItemManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryActionItemManifestStatus::Reopened => {
                return next == PrimaryActionItemManifestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryActionItemManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary action item manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> PrimaryActionItemManifest {
        PrimaryActionItemManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            active: self.active,
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary action item manifest in status open.
pub fn new_primary_action_item_manifest(code: &str, label: &str) -> PrimaryActionItemManifest {
    PrimaryActionItemManifest {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        active: false,
        duration_minutes: 0,
        cluster: "".to_string(),
        pinned: false,
        status: PrimaryActionItemManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_action_item_manifest("code-704", "label-675");
        assert_eq!(item.status, PrimaryActionItemManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_primary_action_item_manifest("code-704", "label-675");
        if item.move_to(PrimaryActionItemManifestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryActionItemManifestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_action_item_manifest("code-704", "label-675");
        item.status = PrimaryActionItemManifestStatus::Archived;
        assert!(
            item.move_to(PrimaryActionItemManifestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
