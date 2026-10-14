use super::*;

use acme_kit::text;

/// Where a rolling quote rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingQuoteRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingQuoteRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingQuoteRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling quote rollup as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteRollup {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub fingerprint: String,
    pub visible: bool,
    pub notes: String,
    pub cluster: String,
    pub phone: String,
    pub summary: String,
    pub status: RollingQuoteRollupStatus,
}

impl RollingQuoteRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingQuoteRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingQuoteRollupStatus) -> bool {
        match self.status {
            RollingQuoteRollupStatus::Proposed => {
                match next {
                    RollingQuoteRollupStatus::Approved | RollingQuoteRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingQuoteRollupStatus::Approved => {
                match next {
                    RollingQuoteRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingQuoteRollupStatus::Rejected => {
                return next == RollingQuoteRollupStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingQuoteRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling quote rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling quote rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> RollingQuoteRollup {
        RollingQuoteRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            visible: self.visible,
            notes: self.notes.clone(),
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling quote rollup in status proposed.
pub fn new_rolling_quote_rollup(code: &str, title: &str) -> RollingQuoteRollup {
    RollingQuoteRollup {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        fingerprint: "".to_string(),
        visible: false,
        notes: "".to_string(),
        cluster: "".to_string(),
        phone: "".to_string(),
        summary: "".to_string(),
        status: RollingQuoteRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_quote_rollup("code-604", "title-541");
        assert_eq!(item.status, RollingQuoteRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rolling_quote_rollup("code-604", "title-541");
        if item.move_to(RollingQuoteRollupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingQuoteRollupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_quote_rollup("code-604", "title-541");
        item.status = RollingQuoteRollupStatus::Withdrawn;
        assert!(
            item.move_to(RollingQuoteRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
