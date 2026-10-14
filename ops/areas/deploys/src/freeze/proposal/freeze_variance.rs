use super::*;

use acme_kit::text;

/// Where a freeze variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezeVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezeVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze variance as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeVariance {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub amount: f64,
    pub ratio: f64,
    pub status: FreezeVarianceStatus,
}

impl FreezeVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeVarianceStatus) -> bool {
        match self.status {
            FreezeVarianceStatus::Proposed => {
                match next {
                    FreezeVarianceStatus::Rejected | FreezeVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeVarianceStatus::Approved => {
                return next == FreezeVarianceStatus::Rejected || next == FreezeVarianceStatus::Withdrawn;
            }
            FreezeVarianceStatus::Rejected => {
                match next {
                    FreezeVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: FreezeVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "freeze variance", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.amount + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeVariance {
        FreezeVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            amount: self.amount,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a freeze variance in status proposed.
pub fn new_freeze_variance(code: &str, label: &str) -> FreezeVariance {
    FreezeVariance {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        amount: 0.0,
        ratio: 0.0,
        status: FreezeVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_freeze_variance("code-27", "label-665");
        assert_eq!(item.status, FreezeVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_freeze_variance("code-27", "label-665");
        if item.move_to(FreezeVarianceStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeVarianceStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_freeze_variance("code-27", "label-665");
        item.status = FreezeVarianceStatus::Withdrawn;
        assert!(
            item.move_to(FreezeVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
