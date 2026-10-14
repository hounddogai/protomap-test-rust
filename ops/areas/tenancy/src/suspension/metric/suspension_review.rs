use super::*;

use acme_kit::text;

/// Where a suspension review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SuspensionReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SuspensionReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReview {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub notes: String,
    pub title: String,
    pub created_at: i64,
    pub phone: String,
    pub source: String,
    pub margin: f64,
    pub minimum: f64,
    pub category: String,
    pub status: SuspensionReviewStatus,
}

impl SuspensionReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionReviewStatus) -> bool {
        match self.status {
            SuspensionReviewStatus::Requested => {
                return next == SuspensionReviewStatus::Granted || next == SuspensionReviewStatus::Revoked;
            }
            SuspensionReviewStatus::Granted => {
                return next == SuspensionReviewStatus::Revoked || next == SuspensionReviewStatus::Expired;
            }
            SuspensionReviewStatus::Revoked => {
                match next {
                    SuspensionReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: SuspensionReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension review {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "suspension review", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.created_at as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionReview {
        SuspensionReview {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            notes: self.notes.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            phone: self.phone.clone(),
            source: self.source.clone(),
            margin: self.margin,
            minimum: self.minimum,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a suspension review in status requested.
pub fn new_suspension_review(code: &str, label: &str) -> SuspensionReview {
    SuspensionReview {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        notes: "".to_string(),
        title: "".to_string(),
        created_at: 0,
        phone: "".to_string(),
        source: "".to_string(),
        margin: 0.0,
        minimum: 0.0,
        category: "".to_string(),
        status: SuspensionReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_suspension_review("code-127", "label-474");
        assert_eq!(item.status, SuspensionReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_suspension_review("code-127", "label-474");
        if item.move_to(SuspensionReviewStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionReviewStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_suspension_review("code-127", "label-474");
        item.status = SuspensionReviewStatus::Expired;
        assert!(
            item.move_to(SuspensionReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
