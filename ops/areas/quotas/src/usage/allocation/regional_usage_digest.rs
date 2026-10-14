use super::*;

use acme_kit::text;

/// Where a regional usage digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalUsageDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalUsageDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalUsageDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional usage digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageDigest {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub retries: i64,
    pub budget: f64,
    pub revision: i64,
    pub segment: String,
    pub limit: i64,
    pub active: bool,
    pub burst: i64,
    pub maximum: f64,
    pub status: RegionalUsageDigestStatus,
}

impl RegionalUsageDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalUsageDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalUsageDigestStatus) -> bool {
        match self.status {
            RegionalUsageDigestStatus::Proposed => {
                return next == RegionalUsageDigestStatus::Approved || next == RegionalUsageDigestStatus::Withdrawn;
            }
            RegionalUsageDigestStatus::Approved => {
                match next {
                    RegionalUsageDigestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalUsageDigestStatus::Rejected => {
                match next {
                    RegionalUsageDigestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalUsageDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional usage digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.maximum * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalUsageDigest {
        RegionalUsageDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            retries: self.retries,
            budget: self.budget,
            revision: self.revision,
            segment: self.segment.clone(),
            limit: self.limit,
            active: self.active,
            burst: self.burst,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a regional usage digest in status proposed.
pub fn new_regional_usage_digest(code: &str, label: &str) -> RegionalUsageDigest {
    RegionalUsageDigest {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        retries: 0,
        budget: 0.0,
        revision: 0,
        segment: "".to_string(),
        limit: 0,
        active: false,
        burst: 0,
        maximum: 0.0,
        status: RegionalUsageDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_usage_digest("code-727", "label-484");
        assert_eq!(item.status, RegionalUsageDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_usage_digest("code-727", "label-484");
        if item.move_to(RegionalUsageDigestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalUsageDigestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_usage_digest("code-727", "label-484");
        item.status = RegionalUsageDigestStatus::Withdrawn;
        assert!(
            item.move_to(RegionalUsageDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
