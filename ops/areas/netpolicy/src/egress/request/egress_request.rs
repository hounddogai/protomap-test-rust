use super::*;

use acme_kit::text;

/// Where a egress request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EgressRequestStatus {
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

impl std::fmt::Display for EgressRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress request as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressRequest {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub score: f64,
    pub reviewer: String,
    pub margin: f64,
    pub region: String,
    pub namespace: String,
    pub discount: f64,
    pub status: EgressRequestStatus,
}

impl EgressRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressRequestStatus) -> bool {
        match self.status {
            EgressRequestStatus::Open => {
                return next == EgressRequestStatus::Archived;
            }
            EgressRequestStatus::InReview => {
                match next {
                    EgressRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRequestStatus::Resolved => {
                match next {
                    EgressRequestStatus::Reopened | EgressRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressRequestStatus::Reopened => {
                return next == EgressRequestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EgressRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "egress request", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.score * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressRequest {
        EgressRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            score: self.score,
            reviewer: self.reviewer.clone(),
            margin: self.margin,
            region: self.region.clone(),
            namespace: self.namespace.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a egress request in status open.
pub fn new_egress_request(code: &str, label: &str) -> EgressRequest {
    EgressRequest {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        score: 0.0,
        reviewer: "".to_string(),
        margin: 0.0,
        region: "".to_string(),
        namespace: "".to_string(),
        discount: 0.0,
        status: EgressRequestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_egress_request("code-189", "label-721");
        assert_eq!(item.status, EgressRequestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_egress_request("code-189", "label-721");
        if item.move_to(EgressRequestStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressRequestStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_egress_request("code-189", "label-721");
        item.status = EgressRequestStatus::Archived;
        assert!(
            item.move_to(EgressRequestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
