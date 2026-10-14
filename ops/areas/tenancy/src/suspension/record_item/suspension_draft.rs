use super::*;

use acme_kit::text;

/// Where a suspension draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SuspensionDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SuspensionDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDraft {
    pub code: String,
    pub title: String,
    pub source: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub segment: String,
    pub discount: f64,
    pub category: String,
    pub contact_email: String,
    pub created_at: i64,
    pub sequence: i64,
    pub status: SuspensionDraftStatus,
}

impl SuspensionDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionDraftStatus) -> bool {
        match self.status {
            SuspensionDraftStatus::Requested => {
                return next == SuspensionDraftStatus::Revoked;
            }
            SuspensionDraftStatus::Granted => {
                match next {
                    SuspensionDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionDraftStatus::Revoked => {
                match next {
                    SuspensionDraftStatus::Expired => {
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

    pub fn move_to(&mut self, next: SuspensionDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension draft {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.discount - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionDraft {
        SuspensionDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            segment: self.segment.clone(),
            discount: self.discount,
            category: self.category.clone(),
            contact_email: self.contact_email.clone(),
            created_at: self.created_at,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension draft in status requested.
pub fn new_suspension_draft(code: &str, title: &str) -> SuspensionDraft {
    SuspensionDraft {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        summary: "".to_string(),
        tags: Vec::new(),
        segment: "".to_string(),
        discount: 0.0,
        category: "".to_string(),
        contact_email: "".to_string(),
        created_at: 0,
        sequence: 0,
        status: SuspensionDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_suspension_draft("code-864", "title-794");
        assert_eq!(item.status, SuspensionDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_suspension_draft("code-864", "title-794");
        if item.move_to(SuspensionDraftStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionDraftStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_suspension_draft("code-864", "title-794");
        item.status = SuspensionDraftStatus::Expired;
        assert!(
            item.move_to(SuspensionDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
