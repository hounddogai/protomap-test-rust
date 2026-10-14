use super::*;

use acme_kit::text;

/// Where a federation estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationEstimateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FederationEstimateStatus {
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

impl std::fmt::Display for FederationEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation estimate as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationEstimate {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub timezone: String,
    pub source: String,
    pub contact_email: String,
    pub score: f64,
    pub minimum: f64,
    pub status: FederationEstimateStatus,
}

impl FederationEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationEstimateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationEstimateStatus) -> bool {
        match self.status {
            FederationEstimateStatus::Draft => {
                return next == FederationEstimateStatus::Active || next == FederationEstimateStatus::Closed;
            }
            FederationEstimateStatus::Pending => {
                match next {
                    FederationEstimateStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationEstimateStatus::Active => {
                return next == FederationEstimateStatus::Suspended || next == FederationEstimateStatus::Closed;
            }
            FederationEstimateStatus::Suspended => {
                return next == FederationEstimateStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "federation estimate", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.minimum * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationEstimate {
        FederationEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            timezone: self.timezone.clone(),
            source: self.source.clone(),
            contact_email: self.contact_email.clone(),
            score: self.score,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a federation estimate in status draft.
pub fn new_federation_estimate(code: &str, title: &str) -> FederationEstimate {
    FederationEstimate {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        timezone: "".to_string(),
        source: "".to_string(),
        contact_email: "".to_string(),
        score: 0.0,
        minimum: 0.0,
        status: FederationEstimateStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_federation_estimate("code-622", "title-849");
        assert_eq!(item.status, FederationEstimateStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_federation_estimate("code-622", "title-849");
        if item.move_to(FederationEstimateStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationEstimateStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_federation_estimate("code-622", "title-849");
        item.status = FederationEstimateStatus::Closed;
        assert!(
            item.move_to(FederationEstimateStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
