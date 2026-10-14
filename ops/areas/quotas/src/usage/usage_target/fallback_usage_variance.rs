use super::*;

use acme_kit::text;

/// Where a fallback usage variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackUsageVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackUsageVarianceStatus {
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

impl std::fmt::Display for FallbackUsageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback usage variance as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackUsageVariance {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub cluster: String,
    pub created_at: i64,
    pub name: String,
    pub namespace: String,
    pub status: FallbackUsageVarianceStatus,
}

impl FallbackUsageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackUsageVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackUsageVarianceStatus) -> bool {
        match self.status {
            FallbackUsageVarianceStatus::Open => {
                match next {
                    FallbackUsageVarianceStatus::InReview | FallbackUsageVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackUsageVarianceStatus::InReview => {
                return next == FallbackUsageVarianceStatus::Resolved || next == FallbackUsageVarianceStatus::Reopened;
            }
            FallbackUsageVarianceStatus::Resolved => {
                return next == FallbackUsageVarianceStatus::Archived;
            }
            FallbackUsageVarianceStatus::Reopened => {
                return next == FallbackUsageVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackUsageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback usage variance {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.created_at as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackUsageVariance {
        FallbackUsageVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            cluster: self.cluster.clone(),
            created_at: self.created_at,
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback usage variance in status open.
pub fn new_fallback_usage_variance(code: &str, label: &str) -> FallbackUsageVariance {
    FallbackUsageVariance {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        cluster: "".to_string(),
        created_at: 0,
        name: "".to_string(),
        namespace: "".to_string(),
        status: FallbackUsageVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_usage_variance("code-759", "label-816");
        assert_eq!(item.status, FallbackUsageVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_usage_variance("code-759", "label-816");
        if item.move_to(FallbackUsageVarianceStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackUsageVarianceStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_usage_variance("code-759", "label-816");
        item.status = FallbackUsageVarianceStatus::Archived;
        assert!(
            item.move_to(FallbackUsageVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
