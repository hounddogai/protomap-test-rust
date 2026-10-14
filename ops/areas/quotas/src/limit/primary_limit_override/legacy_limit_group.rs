use super::*;

use acme_kit::text;

/// Where a legacy limit group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyLimitGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyLimitGroupStatus {
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

impl std::fmt::Display for LegacyLimitGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy limit group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitGroup {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub starts_at: i64,
    pub unit: String,
    pub summary: String,
    pub external_ref: String,
    pub margin: f64,
    pub notes: String,
    pub version: i64,
    pub revision: i64,
    pub status: LegacyLimitGroupStatus,
}

impl LegacyLimitGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyLimitGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyLimitGroupStatus) -> bool {
        match self.status {
            LegacyLimitGroupStatus::Planned => {
                return next == LegacyLimitGroupStatus::Scheduled || next == LegacyLimitGroupStatus::Completed;
            }
            LegacyLimitGroupStatus::Scheduled => {
                match next {
                    LegacyLimitGroupStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyLimitGroupStatus::InProgress => {
                match next {
                    LegacyLimitGroupStatus::Completed | LegacyLimitGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyLimitGroupStatus::Completed => {
                return next == LegacyLimitGroupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyLimitGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy limit group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.starts_at as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyLimitGroup {
        LegacyLimitGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            starts_at: self.starts_at,
            unit: self.unit.clone(),
            summary: self.summary.clone(),
            external_ref: self.external_ref.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            version: self.version,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy limit group in status planned.
pub fn new_legacy_limit_group(code: &str, label: &str) -> LegacyLimitGroup {
    LegacyLimitGroup {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        starts_at: 0,
        unit: "".to_string(),
        summary: "".to_string(),
        external_ref: "".to_string(),
        margin: 0.0,
        notes: "".to_string(),
        version: 0,
        revision: 0,
        status: LegacyLimitGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_limit_group("code-550", "label-712");
        assert_eq!(item.status, LegacyLimitGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_legacy_limit_group("code-550", "label-712");
        if item.move_to(LegacyLimitGroupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyLimitGroupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_limit_group("code-550", "label-712");
        item.status = LegacyLimitGroupStatus::Cancelled;
        assert!(
            item.move_to(LegacyLimitGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
