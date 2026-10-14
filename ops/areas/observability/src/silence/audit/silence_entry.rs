use super::*;

use acme_kit::text;

/// Where a silence entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SilenceEntryStatus {
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

impl std::fmt::Display for SilenceEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEntry {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub title: String,
    pub rate: f64,
    pub description: String,
    pub discount: f64,
    pub source: String,
    pub contact_email: String,
    pub archived: bool,
    pub reviewer: String,
    pub status: SilenceEntryStatus,
}

impl SilenceEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceEntryStatus) -> bool {
        match self.status {
            SilenceEntryStatus::Open => {
                return next == SilenceEntryStatus::Resolved;
            }
            SilenceEntryStatus::InReview => {
                match next {
                    SilenceEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceEntryStatus::Resolved => {
                match next {
                    SilenceEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceEntryStatus::Reopened => {
                return next == SilenceEntryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        (self.rate * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceEntry {
        SilenceEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            title: self.title.clone(),
            rate: self.rate,
            description: self.description.clone(),
            discount: self.discount,
            source: self.source.clone(),
            contact_email: self.contact_email.clone(),
            archived: self.archived,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a silence entry in status open.
pub fn new_silence_entry(code: &str, label: &str) -> SilenceEntry {
    SilenceEntry {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        title: "".to_string(),
        rate: 0.0,
        description: "".to_string(),
        discount: 0.0,
        source: "".to_string(),
        contact_email: "".to_string(),
        archived: false,
        reviewer: "".to_string(),
        status: SilenceEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_silence_entry("code-874", "label-872");
        assert_eq!(item.status, SilenceEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_silence_entry("code-874", "label-872");
        if item.move_to(SilenceEntryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceEntryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_silence_entry("code-874", "label-872");
        item.status = SilenceEntryStatus::Archived;
        assert!(item.move_to(SilenceEntryStatus::Open).is_err(), "expected move to fail");
    }
}
