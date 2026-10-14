use super::*;

use acme_kit::text;

/// Where a suspension tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SuspensionTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SuspensionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTier {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub margin: f64,
    pub window_days: i64,
    pub budget: f64,
    pub issuer: String,
    pub status: SuspensionTierStatus,
}

impl SuspensionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionTierStatus) -> bool {
        match self.status {
            SuspensionTierStatus::Proposed => {
                match next {
                    SuspensionTierStatus::Approved | SuspensionTierStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionTierStatus::Approved => {
                return next == SuspensionTierStatus::Withdrawn;
            }
            SuspensionTierStatus::Rejected => {
                match next {
                    SuspensionTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SuspensionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.window_days as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionTier {
        SuspensionTier {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            margin: self.margin,
            window_days: self.window_days,
            budget: self.budget,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension tier in status proposed.
pub fn new_suspension_tier(code: &str, label: &str) -> SuspensionTier {
    SuspensionTier {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        margin: 0.0,
        window_days: 0,
        budget: 0.0,
        issuer: "".to_string(),
        status: SuspensionTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_suspension_tier("code-119", "label-756");
        assert_eq!(item.status, SuspensionTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_suspension_tier("code-119", "label-756");
        if item.move_to(SuspensionTierStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionTierStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_suspension_tier("code-119", "label-756");
        item.status = SuspensionTierStatus::Withdrawn;
        assert!(
            item.move_to(SuspensionTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
