use super::*;

use acme_kit::text;

/// Where a page mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PageMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PageMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageMapping {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub timezone: String,
    pub visible: bool,
    pub owner_id: String,
    pub contact_email: String,
    pub cipher: String,
    pub status: PageMappingStatus,
}

impl PageMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageMappingStatus) -> bool {
        match self.status {
            PageMappingStatus::Proposed => {
                return next == PageMappingStatus::Approved || next == PageMappingStatus::Withdrawn;
            }
            PageMappingStatus::Approved => {
                return next == PageMappingStatus::Rejected || next == PageMappingStatus::Withdrawn;
            }
            PageMappingStatus::Rejected => {
                return next == PageMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> PageMapping {
        PageMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            timezone: self.timezone.clone(),
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            contact_email: self.contact_email.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a page mapping in status proposed.
pub fn new_page_mapping(code: &str, label: &str) -> PageMapping {
    PageMapping {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        timezone: "".to_string(),
        visible: false,
        owner_id: "".to_string(),
        contact_email: "".to_string(),
        cipher: "".to_string(),
        status: PageMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_page_mapping("code-783", "label-861");
        assert_eq!(item.status, PageMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_page_mapping("code-783", "label-861");
        if item.move_to(PageMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_page_mapping("code-783", "label-861");
        item.status = PageMappingStatus::Withdrawn;
        assert!(
            item.move_to(PageMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
