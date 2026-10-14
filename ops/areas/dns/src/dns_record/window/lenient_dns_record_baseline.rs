use super::*;

use acme_kit::text;

/// Where a lenient dns record baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDnsRecordBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientDnsRecordBaselineStatus {
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

impl std::fmt::Display for LenientDnsRecordBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient dns record baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordBaseline {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub capacity: i64,
    pub summary: String,
    pub duration_minutes: i64,
    pub archived: bool,
    pub status: LenientDnsRecordBaselineStatus,
}

impl LenientDnsRecordBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDnsRecordBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDnsRecordBaselineStatus) -> bool {
        match self.status {
            LenientDnsRecordBaselineStatus::Open => {
                return next == LenientDnsRecordBaselineStatus::InReview
                    || next == LenientDnsRecordBaselineStatus::Archived;
            }
            LenientDnsRecordBaselineStatus::InReview => {
                return next == LenientDnsRecordBaselineStatus::Resolved;
            }
            LenientDnsRecordBaselineStatus::Resolved => {
                match next {
                    LenientDnsRecordBaselineStatus::Reopened | LenientDnsRecordBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDnsRecordBaselineStatus::Reopened => {
                match next {
                    LenientDnsRecordBaselineStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientDnsRecordBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient dns record baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDnsRecordBaseline {
        LenientDnsRecordBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            capacity: self.capacity,
            summary: self.summary.clone(),
            duration_minutes: self.duration_minutes,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient dns record baseline in status open.
pub fn new_lenient_dns_record_baseline(code: &str, label: &str) -> LenientDnsRecordBaseline {
    LenientDnsRecordBaseline {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        capacity: 0,
        summary: "".to_string(),
        duration_minutes: 0,
        archived: false,
        status: LenientDnsRecordBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_dns_record_baseline("code-589", "label-957");
        assert_eq!(item.status, LenientDnsRecordBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_dns_record_baseline("code-589", "label-957");
        if item.move_to(LenientDnsRecordBaselineStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDnsRecordBaselineStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_dns_record_baseline("code-589", "label-957");
        item.status = LenientDnsRecordBaselineStatus::Archived;
        assert!(
            item.move_to(LenientDnsRecordBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
