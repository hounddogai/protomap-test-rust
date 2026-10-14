use super::*;

use acme_kit::text;

/// Where a capped egress variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEgressVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedEgressVarianceStatus {
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

impl std::fmt::Display for CappedEgressVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped egress variance as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEgressVariance {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub locale: String,
    pub maximum: f64,
    pub channel: String,
    pub window_days: i64,
    pub segment: String,
    pub country: String,
    pub status: CappedEgressVarianceStatus,
}

impl CappedEgressVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEgressVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEgressVarianceStatus) -> bool {
        match self.status {
            CappedEgressVarianceStatus::Open => {
                match next {
                    CappedEgressVarianceStatus::InReview | CappedEgressVarianceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEgressVarianceStatus::InReview => {
                return next == CappedEgressVarianceStatus::Resolved || next == CappedEgressVarianceStatus::Archived;
            }
            CappedEgressVarianceStatus::Resolved => {
                return next == CappedEgressVarianceStatus::Reopened || next == CappedEgressVarianceStatus::Archived;
            }
            CappedEgressVarianceStatus::Reopened => {
                return next == CappedEgressVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedEgressVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped egress variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.maximum + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEgressVariance {
        CappedEgressVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            maximum: self.maximum,
            channel: self.channel.clone(),
            window_days: self.window_days,
            segment: self.segment.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a capped egress variance in status open.
pub fn new_capped_egress_variance(code: &str, label: &str) -> CappedEgressVariance {
    CappedEgressVariance {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        locale: "".to_string(),
        maximum: 0.0,
        channel: "".to_string(),
        window_days: 0,
        segment: "".to_string(),
        country: "".to_string(),
        status: CappedEgressVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_egress_variance("code-463", "label-634");
        assert_eq!(item.status, CappedEgressVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_capped_egress_variance("code-463", "label-634");
        if item.move_to(CappedEgressVarianceStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedEgressVarianceStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_egress_variance("code-463", "label-634");
        item.status = CappedEgressVarianceStatus::Archived;
        assert!(
            item.move_to(CappedEgressVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
