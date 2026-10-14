use super::*;

use acme_kit::text;

/// Where a canary preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CanaryPreferenceStatus {
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

impl std::fmt::Display for CanaryPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryPreference {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub grace_days: i64,
    pub unit: String,
    pub status: CanaryPreferenceStatus,
}

impl CanaryPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryPreferenceStatus) -> bool {
        match self.status {
            CanaryPreferenceStatus::Draft => {
                return next == CanaryPreferenceStatus::Suspended;
            }
            CanaryPreferenceStatus::Pending => {
                match next {
                    CanaryPreferenceStatus::Suspended | CanaryPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryPreferenceStatus::Active => {
                return next == CanaryPreferenceStatus::Suspended || next == CanaryPreferenceStatus::Closed;
            }
            CanaryPreferenceStatus::Suspended => {
                match next {
                    CanaryPreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: CanaryPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        (self.budget * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryPreference {
        CanaryPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            grace_days: self.grace_days,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a canary preference in status draft.
pub fn new_canary_preference(code: &str, title: &str) -> CanaryPreference {
    CanaryPreference {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        grace_days: 0,
        unit: "".to_string(),
        status: CanaryPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_canary_preference("code-252", "title-169");
        assert_eq!(item.status, CanaryPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_canary_preference("code-252", "title-169");
        if item.move_to(CanaryPreferenceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryPreferenceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_canary_preference("code-252", "title-169");
        item.status = CanaryPreferenceStatus::Closed;
        assert!(
            item.move_to(CanaryPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
