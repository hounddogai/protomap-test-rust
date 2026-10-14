use super::*;

use acme_kit::text;

/// Where a page adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PageAdjustmentStatus {
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

impl std::fmt::Display for PageAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAdjustment {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub starts_at: i64,
    pub created_at: i64,
    pub status: PageAdjustmentStatus,
}

impl PageAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageAdjustmentStatus) -> bool {
        match self.status {
            PageAdjustmentStatus::Open => {
                match next {
                    PageAdjustmentStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAdjustmentStatus::InReview => {
                match next {
                    PageAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAdjustmentStatus::Resolved => {
                match next {
                    PageAdjustmentStatus::Reopened | PageAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAdjustmentStatus::Reopened => {
                match next {
                    PageAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: PageAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page adjustment {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.starts_at as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageAdjustment {
        PageAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            starts_at: self.starts_at,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a page adjustment in status open.
pub fn new_page_adjustment(code: &str, label: &str) -> PageAdjustment {
    PageAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        starts_at: 0,
        created_at: 0,
        status: PageAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_page_adjustment("code-880", "label-410");
        assert_eq!(item.status, PageAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_page_adjustment("code-880", "label-410");
        if item.move_to(PageAdjustmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageAdjustmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_page_adjustment("code-880", "label-410");
        item.status = PageAdjustmentStatus::Archived;
        assert!(
            item.move_to(PageAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
