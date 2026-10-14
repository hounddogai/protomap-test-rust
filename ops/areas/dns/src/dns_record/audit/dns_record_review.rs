use super::*;

use acme_kit::text;

/// Where a dns record review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DnsRecordReviewStatus {
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

impl std::fmt::Display for DnsRecordReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReview {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub margin: f64,
    pub quota_units: i64,
    pub rotation_days: i64,
    pub active: bool,
    pub attempts: i64,
    pub updated_at: i64,
    pub description: String,
    pub status: DnsRecordReviewStatus,
}

impl DnsRecordReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordReviewStatus) -> bool {
        match self.status {
            DnsRecordReviewStatus::Open => {
                return next == DnsRecordReviewStatus::InReview;
            }
            DnsRecordReviewStatus::InReview => {
                match next {
                    DnsRecordReviewStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordReviewStatus::Resolved => {
                return next == DnsRecordReviewStatus::Reopened || next == DnsRecordReviewStatus::Archived;
            }
            DnsRecordReviewStatus::Reopened => {
                return next == DnsRecordReviewStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record review {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.margin * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordReview {
        DnsRecordReview {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            margin: self.margin,
            quota_units: self.quota_units,
            rotation_days: self.rotation_days,
            active: self.active,
            attempts: self.attempts,
            updated_at: self.updated_at,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record review in status open.
pub fn new_dns_record_review(code: &str, label: &str) -> DnsRecordReview {
    DnsRecordReview {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        margin: 0.0,
        quota_units: 0,
        rotation_days: 0,
        active: false,
        attempts: 0,
        updated_at: 0,
        description: "".to_string(),
        status: DnsRecordReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dns_record_review("code-719", "label-921");
        assert_eq!(item.status, DnsRecordReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_dns_record_review("code-719", "label-921");
        if item.move_to(DnsRecordReviewStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordReviewStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dns_record_review("code-719", "label-921");
        item.status = DnsRecordReviewStatus::Archived;
        assert!(
            item.move_to(DnsRecordReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
