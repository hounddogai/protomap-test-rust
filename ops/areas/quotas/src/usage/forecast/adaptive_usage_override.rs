use super::*;

use acme_kit::text;

/// Where a adaptive usage override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveUsageOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveUsageOverrideStatus {
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

impl std::fmt::Display for AdaptiveUsageOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive usage override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageOverride {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub discount: f64,
    pub rate: f64,
    pub status: AdaptiveUsageOverrideStatus,
}

impl AdaptiveUsageOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveUsageOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveUsageOverrideStatus) -> bool {
        match self.status {
            AdaptiveUsageOverrideStatus::Open => {
                return next == AdaptiveUsageOverrideStatus::Resolved;
            }
            AdaptiveUsageOverrideStatus::InReview => {
                return next == AdaptiveUsageOverrideStatus::Resolved;
            }
            AdaptiveUsageOverrideStatus::Resolved => {
                return next == AdaptiveUsageOverrideStatus::Reopened;
            }
            AdaptiveUsageOverrideStatus::Reopened => {
                match next {
                    AdaptiveUsageOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveUsageOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive usage override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.discount + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveUsageOverride {
        AdaptiveUsageOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            discount: self.discount,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive usage override in status open.
pub fn new_adaptive_usage_override(code: &str, label: &str) -> AdaptiveUsageOverride {
    AdaptiveUsageOverride {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        discount: 0.0,
        rate: 0.0,
        status: AdaptiveUsageOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_usage_override("code-854", "label-112");
        assert_eq!(item.status, AdaptiveUsageOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_adaptive_usage_override("code-854", "label-112");
        if item.move_to(AdaptiveUsageOverrideStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveUsageOverrideStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_usage_override("code-854", "label-112");
        item.status = AdaptiveUsageOverrideStatus::Archived;
        assert!(
            item.move_to(AdaptiveUsageOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
