use super::*;

use acme_kit::text;

/// Where a strict page group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPageGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictPageGroupStatus {
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

impl std::fmt::Display for StrictPageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict page group as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageGroup {
    pub code: String,
    pub label: String,
    pub region: String,
    pub active: bool,
    pub external_ref: String,
    pub starts_at: i64,
    pub status: StrictPageGroupStatus,
}

impl StrictPageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPageGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPageGroupStatus) -> bool {
        match self.status {
            StrictPageGroupStatus::Open => {
                return next == StrictPageGroupStatus::Archived;
            }
            StrictPageGroupStatus::InReview => {
                match next {
                    StrictPageGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPageGroupStatus::Resolved => {
                return next == StrictPageGroupStatus::Archived;
            }
            StrictPageGroupStatus::Reopened => {
                return next == StrictPageGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict page group {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict page group", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> StrictPageGroup {
        StrictPageGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            active: self.active,
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a strict page group in status open.
pub fn new_strict_page_group(code: &str, label: &str) -> StrictPageGroup {
    StrictPageGroup {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        active: false,
        external_ref: "".to_string(),
        starts_at: 0,
        status: StrictPageGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_page_group("code-535", "label-772");
        assert_eq!(item.status, StrictPageGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_strict_page_group("code-535", "label-772");
        if item.move_to(StrictPageGroupStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPageGroupStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_page_group("code-535", "label-772");
        item.status = StrictPageGroupStatus::Archived;
        assert!(
            item.move_to(StrictPageGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
