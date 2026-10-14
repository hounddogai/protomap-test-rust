use super::*;

use acme_kit::text;

/// Where a default redaction template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRedactionTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRedactionTemplateStatus {
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

impl std::fmt::Display for DefaultRedactionTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default redaction template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTemplate {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub expires_at: i64,
    pub archived: bool,
    pub cipher: String,
    pub tags: Vec<String>,
    pub contact_email: String,
    pub unit: String,
    pub status: DefaultRedactionTemplateStatus,
}

impl DefaultRedactionTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRedactionTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRedactionTemplateStatus) -> bool {
        match self.status {
            DefaultRedactionTemplateStatus::Open => {
                return next == DefaultRedactionTemplateStatus::Resolved;
            }
            DefaultRedactionTemplateStatus::InReview => {
                match next {
                    DefaultRedactionTemplateStatus::Resolved | DefaultRedactionTemplateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionTemplateStatus::Resolved => {
                match next {
                    DefaultRedactionTemplateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionTemplateStatus::Reopened => {
                return next == DefaultRedactionTemplateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultRedactionTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default redaction template {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
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
        self.expires_at as f64
    }

    pub fn copy(&self) -> DefaultRedactionTemplate {
        DefaultRedactionTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            archived: self.archived,
            cipher: self.cipher.clone(),
            tags: self.tags.clone(),
            contact_email: self.contact_email.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a default redaction template in status open.
pub fn new_default_redaction_template(code: &str, name: &str) -> DefaultRedactionTemplate {
    DefaultRedactionTemplate {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        expires_at: 0,
        archived: false,
        cipher: "".to_string(),
        tags: Vec::new(),
        contact_email: "".to_string(),
        unit: "".to_string(),
        status: DefaultRedactionTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_redaction_template("code-380", "name-55");
        assert_eq!(item.status, DefaultRedactionTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_redaction_template("code-380", "name-55");
        if item.move_to(DefaultRedactionTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRedactionTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_redaction_template("code-380", "name-55");
        item.status = DefaultRedactionTemplateStatus::Archived;
        assert!(
            item.move_to(DefaultRedactionTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
