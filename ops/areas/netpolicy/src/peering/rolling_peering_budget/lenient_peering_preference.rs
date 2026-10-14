use super::*;

use acme_kit::text;

/// Where a lenient peering preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPeeringPreferenceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientPeeringPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientPeeringPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient peering preference as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreference {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub starts_at: i64,
    pub scope: String,
    pub title: String,
    pub attempts: i64,
    pub reviewer: String,
    pub ratio: f64,
    pub maximum: f64,
    pub rate: f64,
    pub status: LenientPeeringPreferenceStatus,
}

impl LenientPeeringPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPeeringPreferenceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPeeringPreferenceStatus) -> bool {
        match self.status {
            LenientPeeringPreferenceStatus::Planned => {
                match next {
                    LenientPeeringPreferenceStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringPreferenceStatus::Scheduled => {
                match next {
                    LenientPeeringPreferenceStatus::Completed | LenientPeeringPreferenceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringPreferenceStatus::InProgress => {
                match next {
                    LenientPeeringPreferenceStatus::Completed | LenientPeeringPreferenceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringPreferenceStatus::Completed => {
                match next {
                    LenientPeeringPreferenceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientPeeringPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient peering preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ratio * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPeeringPreference {
        LenientPeeringPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            starts_at: self.starts_at,
            scope: self.scope.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            reviewer: self.reviewer.clone(),
            ratio: self.ratio,
            maximum: self.maximum,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient peering preference in status planned.
pub fn new_lenient_peering_preference(code: &str, label: &str) -> LenientPeeringPreference {
    LenientPeeringPreference {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        starts_at: 0,
        scope: "".to_string(),
        title: "".to_string(),
        attempts: 0,
        reviewer: "".to_string(),
        ratio: 0.0,
        maximum: 0.0,
        rate: 0.0,
        status: LenientPeeringPreferenceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_peering_preference("code-453", "label-818");
        assert_eq!(item.status, LenientPeeringPreferenceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_lenient_peering_preference("code-453", "label-818");
        if item.move_to(LenientPeeringPreferenceStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPeeringPreferenceStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_peering_preference("code-453", "label-818");
        item.status = LenientPeeringPreferenceStatus::Cancelled;
        assert!(
            item.move_to(LenientPeeringPreferenceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
