use super::*;

use acme_kit::text;

/// Where a legacy query preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyQueryPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyQueryPreferenceStatus {
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

impl std::fmt::Display for LegacyQueryPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy query preference as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryPreference {
    pub code: String,
    pub label: String,
    pub description: String,
    pub cost: f64,
    pub ceiling: f64,
    pub contact_email: String,
    pub duration_minutes: i64,
    pub scope: String,
    pub rate: f64,
    pub status: LegacyQueryPreferenceStatus,
}

impl LegacyQueryPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyQueryPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyQueryPreferenceStatus) -> bool {
        match self.status {
            LegacyQueryPreferenceStatus::Draft => {
                return next == LegacyQueryPreferenceStatus::Suspended;
            }
            LegacyQueryPreferenceStatus::Pending => {
                return next == LegacyQueryPreferenceStatus::Active || next == LegacyQueryPreferenceStatus::Closed;
            }
            LegacyQueryPreferenceStatus::Active => {
                match next {
                    LegacyQueryPreferenceStatus::Suspended | LegacyQueryPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyQueryPreferenceStatus::Suspended => {
                match next {
                    LegacyQueryPreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyQueryPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy query preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy query preference", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ceiling + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyQueryPreference {
        LegacyQueryPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            cost: self.cost,
            ceiling: self.ceiling,
            contact_email: self.contact_email.clone(),
            duration_minutes: self.duration_minutes,
            scope: self.scope.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy query preference in status draft.
pub fn new_legacy_query_preference(code: &str, label: &str) -> LegacyQueryPreference {
    LegacyQueryPreference {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        cost: 0.0,
        ceiling: 0.0,
        contact_email: "".to_string(),
        duration_minutes: 0,
        scope: "".to_string(),
        rate: 0.0,
        status: LegacyQueryPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_query_preference("code-627", "label-618");
        assert_eq!(item.status, LegacyQueryPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_legacy_query_preference("code-627", "label-618");
        if item.move_to(LegacyQueryPreferenceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyQueryPreferenceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_query_preference("code-627", "label-618");
        item.status = LegacyQueryPreferenceStatus::Closed;
        assert!(
            item.move_to(LegacyQueryPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
