use super::*;

use acme_kit::text;

/// Where a peering summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PeeringSummaryStatus {
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

impl std::fmt::Display for PeeringSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a peering summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringSummary {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub principal: String,
    pub revision: i64,
    pub country: String,
    pub segment: String,
    pub margin: f64,
    pub namespace: String,
    pub source: String,
    pub status: PeeringSummaryStatus,
}

impl PeeringSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringSummaryStatus) -> bool {
        match self.status {
            PeeringSummaryStatus::Open => {
                match next {
                    PeeringSummaryStatus::Resolved | PeeringSummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringSummaryStatus::InReview => {
                return next == PeeringSummaryStatus::Reopened || next == PeeringSummaryStatus::Archived;
            }
            PeeringSummaryStatus::Resolved => {
                match next {
                    PeeringSummaryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringSummaryStatus::Reopened => {
                match next {
                    PeeringSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: PeeringSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering summary", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.margin - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringSummary {
        PeeringSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            revision: self.revision,
            country: self.country.clone(),
            segment: self.segment.clone(),
            margin: self.margin,
            namespace: self.namespace.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a peering summary in status open.
pub fn new_peering_summary(code: &str, name: &str) -> PeeringSummary {
    PeeringSummary {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        principal: "".to_string(),
        revision: 0,
        country: "".to_string(),
        segment: "".to_string(),
        margin: 0.0,
        namespace: "".to_string(),
        source: "".to_string(),
        status: PeeringSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_peering_summary("code-700", "name-516");
        assert_eq!(item.status, PeeringSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_peering_summary("code-700", "name-516");
        if item.move_to(PeeringSummaryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringSummaryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_peering_summary("code-700", "name-516");
        item.status = PeeringSummaryStatus::Archived;
        assert!(
            item.move_to(PeeringSummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
