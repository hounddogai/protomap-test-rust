use super::*;

use acme_kit::text;

/// Where a suspension digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionDigestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SuspensionDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SuspensionDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionDigest {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub contact_email: String,
    pub maximum: f64,
    pub tags: Vec<String>,
    pub key_id: String,
    pub discount: f64,
    pub source: String,
    pub updated_at: i64,
    pub status: SuspensionDigestStatus,
}

impl SuspensionDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionDigestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionDigestStatus) -> bool {
        match self.status {
            SuspensionDigestStatus::Planned => {
                return next == SuspensionDigestStatus::Scheduled || next == SuspensionDigestStatus::Completed;
            }
            SuspensionDigestStatus::Scheduled => {
                match next {
                    SuspensionDigestStatus::Completed | SuspensionDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionDigestStatus::InProgress => {
                match next {
                    SuspensionDigestStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionDigestStatus::Completed => {
                return next == SuspensionDigestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension digest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "suspension digest", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.discount + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionDigest {
        SuspensionDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            maximum: self.maximum,
            tags: self.tags.clone(),
            key_id: self.key_id.clone(),
            discount: self.discount,
            source: self.source.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension digest in status planned.
pub fn new_suspension_digest(code: &str, name: &str) -> SuspensionDigest {
    SuspensionDigest {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        contact_email: "".to_string(),
        maximum: 0.0,
        tags: Vec::new(),
        key_id: "".to_string(),
        discount: 0.0,
        source: "".to_string(),
        updated_at: 0,
        status: SuspensionDigestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_suspension_digest("code-684", "name-867");
        assert_eq!(item.status, SuspensionDigestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_suspension_digest("code-684", "name-867");
        if item.move_to(SuspensionDigestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionDigestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_suspension_digest("code-684", "name-867");
        item.status = SuspensionDigestStatus::Cancelled;
        assert!(
            item.move_to(SuspensionDigestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
