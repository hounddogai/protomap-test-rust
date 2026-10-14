use super::*;

use acme_kit::text;

/// Where a tiered dashboard request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDashboardRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredDashboardRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredDashboardRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered dashboard request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRequest {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub timezone: String,
    pub fingerprint: String,
    pub tags: Vec<String>,
    pub ceiling: f64,
    pub issuer: String,
    pub segment: String,
    pub status: TieredDashboardRequestStatus,
}

impl TieredDashboardRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDashboardRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDashboardRequestStatus) -> bool {
        match self.status {
            TieredDashboardRequestStatus::Proposed => {
                match next {
                    TieredDashboardRequestStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDashboardRequestStatus::Approved => {
                match next {
                    TieredDashboardRequestStatus::Rejected | TieredDashboardRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDashboardRequestStatus::Rejected => {
                match next {
                    TieredDashboardRequestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredDashboardRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered dashboard request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered dashboard request", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.minimum + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDashboardRequest {
        TieredDashboardRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            tags: self.tags.clone(),
            ceiling: self.ceiling,
            issuer: self.issuer.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered dashboard request in status proposed.
pub fn new_tiered_dashboard_request(code: &str, title: &str) -> TieredDashboardRequest {
    TieredDashboardRequest {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        tags: Vec::new(),
        ceiling: 0.0,
        issuer: "".to_string(),
        segment: "".to_string(),
        status: TieredDashboardRequestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_dashboard_request("code-402", "title-234");
        assert_eq!(item.status, TieredDashboardRequestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_dashboard_request("code-402", "title-234");
        if item.move_to(TieredDashboardRequestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDashboardRequestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_dashboard_request("code-402", "title-234");
        item.status = TieredDashboardRequestStatus::Withdrawn;
        assert!(
            item.move_to(TieredDashboardRequestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
