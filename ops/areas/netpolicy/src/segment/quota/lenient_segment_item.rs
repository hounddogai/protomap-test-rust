use super::*;

use acme_kit::text;

/// Where a lenient segment item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSegmentItemStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientSegmentItemStatus {
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

impl std::fmt::Display for LenientSegmentItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient segment item as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSegmentItem {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub ttl_seconds: i64,
    pub active: bool,
    pub label: String,
    pub tags: Vec<String>,
    pub phone: String,
    pub created_at: i64,
    pub audience: String,
    pub status: LenientSegmentItemStatus,
}

impl LenientSegmentItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSegmentItemStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSegmentItemStatus) -> bool {
        match self.status {
            LenientSegmentItemStatus::Open => {
                return next == LenientSegmentItemStatus::Resolved;
            }
            LenientSegmentItemStatus::InReview => {
                match next {
                    LenientSegmentItemStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSegmentItemStatus::Resolved => {
                return next == LenientSegmentItemStatus::Reopened || next == LenientSegmentItemStatus::Archived;
            }
            LenientSegmentItemStatus::Reopened => {
                match next {
                    LenientSegmentItemStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientSegmentItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient segment item {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientSegmentItem {
        LenientSegmentItem {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            ttl_seconds: self.ttl_seconds,
            active: self.active,
            label: self.label.clone(),
            tags: self.tags.clone(),
            phone: self.phone.clone(),
            created_at: self.created_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient segment item in status open.
pub fn new_lenient_segment_item(code: &str, title: &str) -> LenientSegmentItem {
    LenientSegmentItem {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        ttl_seconds: 0,
        active: false,
        label: "".to_string(),
        tags: Vec::new(),
        phone: "".to_string(),
        created_at: 0,
        audience: "".to_string(),
        status: LenientSegmentItemStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_segment_item("code-973", "title-87");
        assert_eq!(item.status, LenientSegmentItemStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_lenient_segment_item("code-973", "title-87");
        if item.move_to(LenientSegmentItemStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSegmentItemStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_segment_item("code-973", "title-87");
        item.status = LenientSegmentItemStatus::Archived;
        assert!(
            item.move_to(LenientSegmentItemStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
