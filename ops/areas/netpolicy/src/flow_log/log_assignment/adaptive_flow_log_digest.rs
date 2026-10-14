use super::*;

use acme_kit::text;

/// Where a adaptive flow log digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFlowLogDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveFlowLogDigestStatus {
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

impl std::fmt::Display for AdaptiveFlowLogDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive flow log digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogDigest {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub fingerprint: String,
    pub scope: String,
    pub minimum: f64,
    pub updated_at: i64,
    pub status: AdaptiveFlowLogDigestStatus,
}

impl AdaptiveFlowLogDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFlowLogDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFlowLogDigestStatus) -> bool {
        match self.status {
            AdaptiveFlowLogDigestStatus::Draft => {
                return next == AdaptiveFlowLogDigestStatus::Active || next == AdaptiveFlowLogDigestStatus::Suspended;
            }
            AdaptiveFlowLogDigestStatus::Pending => {
                match next {
                    AdaptiveFlowLogDigestStatus::Suspended | AdaptiveFlowLogDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFlowLogDigestStatus::Active => {
                return next == AdaptiveFlowLogDigestStatus::Closed;
            }
            AdaptiveFlowLogDigestStatus::Suspended => {
                match next {
                    AdaptiveFlowLogDigestStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveFlowLogDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive flow log digest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.minimum + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFlowLogDigest {
        AdaptiveFlowLogDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            fingerprint: self.fingerprint.clone(),
            scope: self.scope.clone(),
            minimum: self.minimum,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive flow log digest in status draft.
pub fn new_adaptive_flow_log_digest(code: &str, title: &str) -> AdaptiveFlowLogDigest {
    AdaptiveFlowLogDigest {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        fingerprint: "".to_string(),
        scope: "".to_string(),
        minimum: 0.0,
        updated_at: 0,
        status: AdaptiveFlowLogDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_adaptive_flow_log_digest("code-420", "title-16");
        assert_eq!(item.status, AdaptiveFlowLogDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_adaptive_flow_log_digest("code-420", "title-16");
        if item.move_to(AdaptiveFlowLogDigestStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveFlowLogDigestStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_adaptive_flow_log_digest("code-420", "title-16");
        item.status = AdaptiveFlowLogDigestStatus::Closed;
        assert!(
            item.move_to(AdaptiveFlowLogDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
