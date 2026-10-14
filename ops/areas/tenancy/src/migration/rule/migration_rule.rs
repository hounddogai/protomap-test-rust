use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a migration rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MigrationRuleStatus {
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

impl std::fmt::Display for MigrationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a migration rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRule {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub approved: bool,
    pub minimum: f64,
    pub locale: String,
    pub status: MigrationRuleStatus,
}

impl MigrationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationRuleStatus) -> bool {
        match self.status {
            MigrationRuleStatus::Planned => {
                return next == MigrationRuleStatus::Cancelled;
            }
            MigrationRuleStatus::Scheduled => {
                match next {
                    MigrationRuleStatus::InProgress | MigrationRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationRuleStatus::InProgress => {
                match next {
                    MigrationRuleStatus::Completed | MigrationRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationRuleStatus::Completed => {
                return next == MigrationRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ceiling - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationRule {
        MigrationRule {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            approved: self.approved,
            minimum: self.minimum,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a migration rule in status planned.
pub fn new_migration_rule(code: &str, title: &str) -> MigrationRule {
    MigrationRule {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        approved: false,
        minimum: 0.0,
        locale: "".to_string(),
        status: MigrationRuleStatus::Planned,
    }
}
