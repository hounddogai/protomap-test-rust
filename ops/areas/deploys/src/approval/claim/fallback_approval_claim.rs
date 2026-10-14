use super::*;

use acme_kit::text;

/// Where a fallback approval claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackApprovalClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackApprovalClaimStatus {
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

impl std::fmt::Display for FallbackApprovalClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback approval claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackApprovalClaim {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub limit: i64,
    pub label: String,
    pub scope: String,
    pub maximum: f64,
    pub source: String,
    pub updated_at: i64,
    pub key_id: String,
    pub status: FallbackApprovalClaimStatus,
}

impl FallbackApprovalClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackApprovalClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackApprovalClaimStatus) -> bool {
        match self.status {
            FallbackApprovalClaimStatus::Draft => {
                match next {
                    FallbackApprovalClaimStatus::Active | FallbackApprovalClaimStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackApprovalClaimStatus::Pending => {
                return next == FallbackApprovalClaimStatus::Active || next == FallbackApprovalClaimStatus::Closed;
            }
            FallbackApprovalClaimStatus::Active => {
                match next {
                    FallbackApprovalClaimStatus::Suspended | FallbackApprovalClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackApprovalClaimStatus::Suspended => {
                return next == FallbackApprovalClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackApprovalClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback approval claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback approval claim", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.updated_at as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackApprovalClaim {
        FallbackApprovalClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            limit: self.limit,
            label: self.label.clone(),
            scope: self.scope.clone(),
            maximum: self.maximum,
            source: self.source.clone(),
            updated_at: self.updated_at,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback approval claim in status draft.
pub fn new_fallback_approval_claim(code: &str, title: &str) -> FallbackApprovalClaim {
    FallbackApprovalClaim {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        limit: 0,
        label: "".to_string(),
        scope: "".to_string(),
        maximum: 0.0,
        source: "".to_string(),
        updated_at: 0,
        key_id: "".to_string(),
        status: FallbackApprovalClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_approval_claim("code-38", "title-509");
        assert_eq!(item.status, FallbackApprovalClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_fallback_approval_claim("code-38", "title-509");
        if item.move_to(FallbackApprovalClaimStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackApprovalClaimStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_approval_claim("code-38", "title-509");
        item.status = FallbackApprovalClaimStatus::Closed;
        assert!(
            item.move_to(FallbackApprovalClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
