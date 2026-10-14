use super::*;

use acme_kit::text;

/// Where a canary tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanaryTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanaryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTier {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub approved: bool,
    pub cipher: String,
    pub source: String,
    pub cost: f64,
    pub revision: i64,
    pub status: CanaryTierStatus,
}

impl CanaryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryTierStatus) -> bool {
        match self.status {
            CanaryTierStatus::Proposed => {
                match next {
                    CanaryTierStatus::Rejected | CanaryTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryTierStatus::Approved => {
                match next {
                    CanaryTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryTierStatus::Rejected => {
                return next == CanaryTierStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "canary tier", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.cost + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryTier {
        CanaryTier {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            approved: self.approved,
            cipher: self.cipher.clone(),
            source: self.source.clone(),
            cost: self.cost,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a canary tier in status proposed.
pub fn new_canary_tier(code: &str, title: &str) -> CanaryTier {
    CanaryTier {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        approved: false,
        cipher: "".to_string(),
        source: "".to_string(),
        cost: 0.0,
        revision: 0,
        status: CanaryTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_canary_tier("code-329", "title-860");
        assert_eq!(item.status, CanaryTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_canary_tier("code-329", "title-860");
        if item.move_to(CanaryTierStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryTierStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_canary_tier("code-329", "title-860");
        item.status = CanaryTierStatus::Withdrawn;
        assert!(
            item.move_to(CanaryTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
