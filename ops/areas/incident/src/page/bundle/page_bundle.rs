use super::*;

use acme_kit::text;

/// Where a page bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PageBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PageBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page bundle as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageBundle {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub limit: i64,
    pub summary: String,
    pub rate: f64,
    pub rotation_days: i64,
    pub retries: i64,
    pub label: String,
    pub window_days: i64,
    pub status: PageBundleStatus,
}

impl PageBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageBundleStatus) -> bool {
        match self.status {
            PageBundleStatus::Proposed => {
                match next {
                    PageBundleStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageBundleStatus::Approved => {
                return next == PageBundleStatus::Rejected;
            }
            PageBundleStatus::Rejected => {
                return next == PageBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rotation_days as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageBundle {
        PageBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            limit: self.limit,
            summary: self.summary.clone(),
            rate: self.rate,
            rotation_days: self.rotation_days,
            retries: self.retries,
            label: self.label.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a page bundle in status proposed.
pub fn new_page_bundle(code: &str, name: &str) -> PageBundle {
    PageBundle {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        limit: 0,
        summary: "".to_string(),
        rate: 0.0,
        rotation_days: 0,
        retries: 0,
        label: "".to_string(),
        window_days: 0,
        status: PageBundleStatus::Proposed,
    }
}
