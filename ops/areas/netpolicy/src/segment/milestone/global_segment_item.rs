use super::*;

use acme_kit::text;

/// Where a global segment item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSegmentItemStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalSegmentItemStatus {
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

impl std::fmt::Display for GlobalSegmentItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global segment item as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentItem {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub quota_units: i64,
    pub description: String,
    pub port: i64,
    pub starts_at: i64,
    pub margin: f64,
    pub status: GlobalSegmentItemStatus,
}

impl GlobalSegmentItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSegmentItemStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSegmentItemStatus) -> bool {
        match self.status {
            GlobalSegmentItemStatus::Open => {
                return next == GlobalSegmentItemStatus::InReview;
            }
            GlobalSegmentItemStatus::InReview => {
                match next {
                    GlobalSegmentItemStatus::Resolved | GlobalSegmentItemStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentItemStatus::Resolved => {
                return next == GlobalSegmentItemStatus::Archived;
            }
            GlobalSegmentItemStatus::Reopened => {
                return next == GlobalSegmentItemStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSegmentItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global segment item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "global segment item", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.starts_at as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSegmentItem {
        GlobalSegmentItem {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            quota_units: self.quota_units,
            description: self.description.clone(),
            port: self.port,
            starts_at: self.starts_at,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global segment item in status open.
pub fn new_global_segment_item(code: &str, name: &str) -> GlobalSegmentItem {
    GlobalSegmentItem {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        quota_units: 0,
        description: "".to_string(),
        port: 0,
        starts_at: 0,
        margin: 0.0,
        status: GlobalSegmentItemStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_segment_item("code-725", "name-186");
        assert_eq!(item.status, GlobalSegmentItemStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_global_segment_item("code-725", "name-186");
        if item.move_to(GlobalSegmentItemStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSegmentItemStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_segment_item("code-725", "name-186");
        item.status = GlobalSegmentItemStatus::Archived;
        assert!(
            item.move_to(GlobalSegmentItemStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
