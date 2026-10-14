use super::*;

use acme_kit::text;

/// Where a lenient credential segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCredentialSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientCredentialSegmentStatus {
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

impl std::fmt::Display for LenientCredentialSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient credential segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialSegment {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub quota_units: i64,
    pub channel: String,
    pub rate: f64,
    pub quantity: i64,
    pub locale: String,
    pub port: i64,
    pub contact_email: String,
    pub status: LenientCredentialSegmentStatus,
}

impl LenientCredentialSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCredentialSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCredentialSegmentStatus) -> bool {
        match self.status {
            LenientCredentialSegmentStatus::Open => {
                return next == LenientCredentialSegmentStatus::InReview
                    || next == LenientCredentialSegmentStatus::Resolved;
            }
            LenientCredentialSegmentStatus::InReview => {
                match next {
                    LenientCredentialSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCredentialSegmentStatus::Resolved => {
                match next {
                    LenientCredentialSegmentStatus::Reopened | LenientCredentialSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCredentialSegmentStatus::Reopened => {
                return next == LenientCredentialSegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientCredentialSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient credential segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quantity as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCredentialSegment {
        LenientCredentialSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            quota_units: self.quota_units,
            channel: self.channel.clone(),
            rate: self.rate,
            quantity: self.quantity,
            locale: self.locale.clone(),
            port: self.port,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient credential segment in status open.
pub fn new_lenient_credential_segment(code: &str, label: &str) -> LenientCredentialSegment {
    LenientCredentialSegment {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        quota_units: 0,
        channel: "".to_string(),
        rate: 0.0,
        quantity: 0,
        locale: "".to_string(),
        port: 0,
        contact_email: "".to_string(),
        status: LenientCredentialSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_credential_segment("code-172", "label-456");
        assert_eq!(item.status, LenientCredentialSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_credential_segment("code-172", "label-456");
        if item.move_to(LenientCredentialSegmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientCredentialSegmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_credential_segment("code-172", "label-456");
        item.status = LenientCredentialSegmentStatus::Archived;
        assert!(
            item.move_to(LenientCredentialSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
