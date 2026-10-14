use super::*;

use acme_kit::text;

/// Where a segment claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentClaimStatus {
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

impl std::fmt::Display for SegmentClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentClaim {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub burst: i64,
    pub locale: String,
    pub ttl_seconds: i64,
    pub reviewer: String,
    pub status: SegmentClaimStatus,
}

impl SegmentClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentClaimStatus) -> bool {
        match self.status {
            SegmentClaimStatus::Draft => {
                return next == SegmentClaimStatus::Pending;
            }
            SegmentClaimStatus::Pending => {
                return next == SegmentClaimStatus::Suspended || next == SegmentClaimStatus::Closed;
            }
            SegmentClaimStatus::Active => {
                return next == SegmentClaimStatus::Suspended || next == SegmentClaimStatus::Closed;
            }
            SegmentClaimStatus::Suspended => {
                return next == SegmentClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentClaim {
        SegmentClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            burst: self.burst,
            locale: self.locale.clone(),
            ttl_seconds: self.ttl_seconds,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a segment claim in status draft.
pub fn new_segment_claim(code: &str, title: &str) -> SegmentClaim {
    SegmentClaim {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        burst: 0,
        locale: "".to_string(),
        ttl_seconds: 0,
        reviewer: "".to_string(),
        status: SegmentClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_segment_claim("code-264", "title-320");
        assert_eq!(item.status, SegmentClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_segment_claim("code-264", "title-320");
        if item.move_to(SegmentClaimStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentClaimStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_segment_claim("code-264", "title-320");
        item.status = SegmentClaimStatus::Closed;
        assert!(
            item.move_to(SegmentClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
