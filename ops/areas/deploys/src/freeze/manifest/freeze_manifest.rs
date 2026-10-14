use super::*;

use acme_kit::text;

/// Where a freeze manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeManifestStatus {
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

impl std::fmt::Display for FreezeManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a freeze manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeManifest {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub starts_at: i64,
    pub amount: f64,
    pub cost: f64,
    pub visible: bool,
    pub pinned: bool,
    pub status: FreezeManifestStatus,
}

impl FreezeManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeManifestStatus) -> bool {
        match self.status {
            FreezeManifestStatus::Open => {
                match next {
                    FreezeManifestStatus::InReview | FreezeManifestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeManifestStatus::InReview => {
                return next == FreezeManifestStatus::Resolved || next == FreezeManifestStatus::Reopened;
            }
            FreezeManifestStatus::Resolved => {
                return next == FreezeManifestStatus::Reopened || next == FreezeManifestStatus::Archived;
            }
            FreezeManifestStatus::Reopened => {
                return next == FreezeManifestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.cost - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeManifest {
        FreezeManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            starts_at: self.starts_at,
            amount: self.amount,
            cost: self.cost,
            visible: self.visible,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a freeze manifest in status open.
pub fn new_freeze_manifest(code: &str, title: &str) -> FreezeManifest {
    FreezeManifest {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        starts_at: 0,
        amount: 0.0,
        cost: 0.0,
        visible: false,
        pinned: false,
        status: FreezeManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_freeze_manifest("code-844", "title-276");
        assert_eq!(item.status, FreezeManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_freeze_manifest("code-844", "title-276");
        if item.move_to(FreezeManifestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeManifestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_freeze_manifest("code-844", "title-276");
        item.status = FreezeManifestStatus::Archived;
        assert!(
            item.move_to(FreezeManifestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
