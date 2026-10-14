use super::*;

use acme_kit::text;

/// Where a legacy rule preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRulePreferenceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyRulePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LegacyRulePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy rule preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRulePreference {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub locale: String,
    pub approved: bool,
    pub window_days: i64,
    pub region: String,
    pub segment: String,
    pub status: LegacyRulePreferenceStatus,
}

impl LegacyRulePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRulePreferenceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRulePreferenceStatus) -> bool {
        match self.status {
            LegacyRulePreferenceStatus::New => {
                match next {
                    LegacyRulePreferenceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRulePreferenceStatus::Queued => {
                match next {
                    LegacyRulePreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRulePreferenceStatus::Running => {
                match next {
                    LegacyRulePreferenceStatus::Done | LegacyRulePreferenceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRulePreferenceStatus::Done => {
                return next == LegacyRulePreferenceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRulePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rule preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.window_days as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRulePreference {
        LegacyRulePreference {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            locale: self.locale.clone(),
            approved: self.approved,
            window_days: self.window_days,
            region: self.region.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a legacy rule preference in status new.
pub fn new_legacy_rule_preference(code: &str, name: &str) -> LegacyRulePreference {
    LegacyRulePreference {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        locale: "".to_string(),
        approved: false,
        window_days: 0,
        region: "".to_string(),
        segment: "".to_string(),
        status: LegacyRulePreferenceStatus::New,
    }
}
