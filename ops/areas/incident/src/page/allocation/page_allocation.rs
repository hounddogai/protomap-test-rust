use super::*;

use acme_kit::text;

/// Where a page allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PageAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for PageAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAllocation {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub phone: String,
    pub label: String,
    pub scope: String,
    pub source: String,
    pub limit: i64,
    pub checksum: String,
    pub owner_id: String,
    pub version: i64,
    pub status: PageAllocationStatus,
}

impl PageAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageAllocationStatus) -> bool {
        match self.status {
            PageAllocationStatus::Draft => {
                return next == PageAllocationStatus::Pending || next == PageAllocationStatus::Active;
            }
            PageAllocationStatus::Pending => {
                return next == PageAllocationStatus::Active || next == PageAllocationStatus::Closed;
            }
            PageAllocationStatus::Active => {
                return next == PageAllocationStatus::Closed;
            }
            PageAllocationStatus::Suspended => {
                match next {
                    PageAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: PageAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.version as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageAllocation {
        PageAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            phone: self.phone.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            source: self.source.clone(),
            limit: self.limit,
            checksum: self.checksum.clone(),
            owner_id: self.owner_id.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a page allocation in status draft.
pub fn new_page_allocation(code: &str, title: &str) -> PageAllocation {
    PageAllocation {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        phone: "".to_string(),
        label: "".to_string(),
        scope: "".to_string(),
        source: "".to_string(),
        limit: 0,
        checksum: "".to_string(),
        owner_id: "".to_string(),
        version: 0,
        status: PageAllocationStatus::Draft,
    }
}
