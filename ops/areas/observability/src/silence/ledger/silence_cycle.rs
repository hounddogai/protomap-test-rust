use super::*;

use acme_kit::text;

/// Where a silence cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SilenceCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SilenceCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCycle {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub ends_at: i64,
    pub budget: f64,
    pub active: bool,
    pub notes: String,
    pub margin: f64,
    pub retries: i64,
    pub status: SilenceCycleStatus,
}

impl SilenceCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceCycleStatus) -> bool {
        match self.status {
            SilenceCycleStatus::Proposed => {
                match next {
                    SilenceCycleStatus::Approved | SilenceCycleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceCycleStatus::Approved => {
                match next {
                    SilenceCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceCycleStatus::Rejected => {
                return next == SilenceCycleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence cycle", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceCycle {
        SilenceCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            ends_at: self.ends_at,
            budget: self.budget,
            active: self.active,
            notes: self.notes.clone(),
            margin: self.margin,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence cycle in status proposed.
pub fn new_silence_cycle(code: &str, label: &str) -> SilenceCycle {
    SilenceCycle {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        ends_at: 0,
        budget: 0.0,
        active: false,
        notes: "".to_string(),
        margin: 0.0,
        retries: 0,
        status: SilenceCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_silence_cycle("code-992", "label-155");
        assert_eq!(item.status, SilenceCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_silence_cycle("code-992", "label-155");
        if item.move_to(SilenceCycleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceCycleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_silence_cycle("code-992", "label-155");
        item.status = SilenceCycleStatus::Withdrawn;
        assert!(
            item.move_to(SilenceCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
