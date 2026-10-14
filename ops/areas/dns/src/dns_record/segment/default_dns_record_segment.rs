use super::*;

use acme_kit::text;

/// Where a default dns record segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDnsRecordSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultDnsRecordSegmentStatus {
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

impl std::fmt::Display for DefaultDnsRecordSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default dns record segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordSegment {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub active: bool,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub floor_value: f64,
    pub amount: f64,
    pub reviewer: String,
    pub status: DefaultDnsRecordSegmentStatus,
}

impl DefaultDnsRecordSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDnsRecordSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDnsRecordSegmentStatus) -> bool {
        match self.status {
            DefaultDnsRecordSegmentStatus::Open => {
                match next {
                    DefaultDnsRecordSegmentStatus::InReview | DefaultDnsRecordSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDnsRecordSegmentStatus::InReview => {
                return next == DefaultDnsRecordSegmentStatus::Resolved;
            }
            DefaultDnsRecordSegmentStatus::Resolved => {
                match next {
                    DefaultDnsRecordSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDnsRecordSegmentStatus::Reopened => {
                match next {
                    DefaultDnsRecordSegmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultDnsRecordSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default dns record segment {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.floor_value - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDnsRecordSegment {
        DefaultDnsRecordSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            active: self.active,
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            floor_value: self.floor_value,
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default dns record segment in status open.
pub fn new_default_dns_record_segment(code: &str, name: &str) -> DefaultDnsRecordSegment {
    DefaultDnsRecordSegment {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        active: false,
        tags: Vec::new(),
        fingerprint: "".to_string(),
        floor_value: 0.0,
        amount: 0.0,
        reviewer: "".to_string(),
        status: DefaultDnsRecordSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_dns_record_segment("code-209", "name-901");
        assert_eq!(item.status, DefaultDnsRecordSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_default_dns_record_segment("code-209", "name-901");
        if item.move_to(DefaultDnsRecordSegmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDnsRecordSegmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_dns_record_segment("code-209", "name-901");
        item.status = DefaultDnsRecordSegmentStatus::Archived;
        assert!(
            item.move_to(DefaultDnsRecordSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
