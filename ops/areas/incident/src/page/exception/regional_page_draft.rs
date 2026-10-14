use super::*;

use acme_kit::text;

/// Where a regional page draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPageDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalPageDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RegionalPageDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional page draft as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraft {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub name: String,
    pub principal: String,
    pub currency: String,
    pub tags: Vec<String>,
    pub status: RegionalPageDraftStatus,
}

impl RegionalPageDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPageDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPageDraftStatus) -> bool {
        match self.status {
            RegionalPageDraftStatus::Idle => {
                return next == RegionalPageDraftStatus::Ready;
            }
            RegionalPageDraftStatus::Warming => {
                match next {
                    RegionalPageDraftStatus::Ready | RegionalPageDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPageDraftStatus::Ready => {
                match next {
                    RegionalPageDraftStatus::Draining | RegionalPageDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPageDraftStatus::Draining => {
                match next {
                    RegionalPageDraftStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalPageDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional page draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> RegionalPageDraft {
        RegionalPageDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            name: self.name.clone(),
            principal: self.principal.clone(),
            currency: self.currency.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional page draft in status idle.
pub fn new_regional_page_draft(code: &str, title: &str) -> RegionalPageDraft {
    RegionalPageDraft {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        name: "".to_string(),
        principal: "".to_string(),
        currency: "".to_string(),
        tags: Vec::new(),
        status: RegionalPageDraftStatus::Idle,
    }
}
