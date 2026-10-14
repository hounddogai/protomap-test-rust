use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a regional postmortem variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPostmortemVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalPostmortemVarianceStatus {
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

impl std::fmt::Display for RegionalPostmortemVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional postmortem variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPostmortemVariance {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub country: String,
    pub expires_at: i64,
    pub status: RegionalPostmortemVarianceStatus,
}

impl RegionalPostmortemVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPostmortemVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPostmortemVarianceStatus) -> bool {
        match self.status {
            RegionalPostmortemVarianceStatus::Planned => {
                match next {
                    RegionalPostmortemVarianceStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalPostmortemVarianceStatus::Scheduled => {
                return next == RegionalPostmortemVarianceStatus::InProgress;
            }
            RegionalPostmortemVarianceStatus::InProgress => {
                return next == RegionalPostmortemVarianceStatus::Cancelled;
            }
            RegionalPostmortemVarianceStatus::Completed => {
                match next {
                    RegionalPostmortemVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalPostmortemVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional postmortem variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional postmortem variance", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> RegionalPostmortemVariance {
        RegionalPostmortemVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            country: self.country.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional postmortem variance in status planned.
pub fn new_regional_postmortem_variance(code: &str, name: &str) -> RegionalPostmortemVariance {
    RegionalPostmortemVariance {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        attributes: BTreeMap::new(),
        category: "".to_string(),
        country: "".to_string(),
        expires_at: 0,
        status: RegionalPostmortemVarianceStatus::Planned,
    }
}
