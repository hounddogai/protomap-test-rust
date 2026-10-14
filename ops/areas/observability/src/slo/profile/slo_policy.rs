use super::*;

use acme_kit::text;

/// Where a slo policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SloPolicyStatus {
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

impl std::fmt::Display for SloPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloPolicy {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub region: String,
    pub margin: f64,
    pub status: SloPolicyStatus,
}

impl SloPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloPolicyStatus) -> bool {
        match self.status {
            SloPolicyStatus::Open => {
                match next {
                    SloPolicyStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloPolicyStatus::InReview => {
                return next == SloPolicyStatus::Resolved || next == SloPolicyStatus::Reopened;
            }
            SloPolicyStatus::Resolved => {
                return next == SloPolicyStatus::Reopened;
            }
            SloPolicyStatus::Reopened => {
                match next {
                    SloPolicyStatus::Archived => {
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

    pub fn move_to(&mut self, next: SloPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "slo policy", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.limit as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloPolicy {
        SloPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            region: self.region.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a slo policy in status open.
pub fn new_slo_policy(code: &str, label: &str) -> SloPolicy {
    SloPolicy {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        region: "".to_string(),
        margin: 0.0,
        status: SloPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_slo_policy("code-695", "label-861");
        assert_eq!(item.status, SloPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_slo_policy("code-695", "label-861");
        if item.move_to(SloPolicyStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloPolicyStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_slo_policy("code-695", "label-861");
        item.status = SloPolicyStatus::Archived;
        assert!(item.move_to(SloPolicyStatus::Open).is_err(), "expected move to fail");
    }
}
