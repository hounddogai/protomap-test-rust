use super::*;

use acme_kit::text;

/// Where a lenient dashboard review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDashboardReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientDashboardReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LenientDashboardReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient dashboard review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardReview {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub audience: String,
    pub namespace: String,
    pub fingerprint: String,
    pub port: i64,
    pub status: LenientDashboardReviewStatus,
}

impl LenientDashboardReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDashboardReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDashboardReviewStatus) -> bool {
        match self.status {
            LenientDashboardReviewStatus::Draft => {
                match next {
                    LenientDashboardReviewStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDashboardReviewStatus::Pending => {
                match next {
                    LenientDashboardReviewStatus::Active | LenientDashboardReviewStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDashboardReviewStatus::Active => {
                match next {
                    LenientDashboardReviewStatus::Suspended | LenientDashboardReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDashboardReviewStatus::Suspended => {
                match next {
                    LenientDashboardReviewStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientDashboardReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient dashboard review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient dashboard review", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> LenientDashboardReview {
        LenientDashboardReview {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            audience: self.audience.clone(),
            namespace: self.namespace.clone(),
            fingerprint: self.fingerprint.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient dashboard review in status draft.
pub fn new_lenient_dashboard_review(code: &str, name: &str) -> LenientDashboardReview {
    LenientDashboardReview {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        audience: "".to_string(),
        namespace: "".to_string(),
        fingerprint: "".to_string(),
        port: 0,
        status: LenientDashboardReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_dashboard_review("code-258", "name-209");
        assert_eq!(item.status, LenientDashboardReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_dashboard_review("code-258", "name-209");
        if item.move_to(LenientDashboardReviewStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDashboardReviewStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_dashboard_review("code-258", "name-209");
        item.status = LenientDashboardReviewStatus::Closed;
        assert!(
            item.move_to(LenientDashboardReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
