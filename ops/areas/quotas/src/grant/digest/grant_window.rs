use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a grant window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GrantWindowStatus {
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

impl std::fmt::Display for GrantWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantWindow {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub attributes: BTreeMap<String, String>,
    pub minimum: f64,
    pub tenant_id: String,
    pub issuer: String,
    pub source: String,
    pub ceiling: f64,
    pub principal: String,
    pub status: GrantWindowStatus,
}

impl GrantWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantWindowStatus) -> bool {
        match self.status {
            GrantWindowStatus::Planned => {
                return next == GrantWindowStatus::InProgress;
            }
            GrantWindowStatus::Scheduled => {
                match next {
                    GrantWindowStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantWindowStatus::InProgress => {
                return next == GrantWindowStatus::Cancelled;
            }
            GrantWindowStatus::Completed => {
                return next == GrantWindowStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant window {} cannot move from {} to {}",
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
        ((self.minimum - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantWindow {
        GrantWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            attributes: self.attributes.clone(),
            minimum: self.minimum,
            tenant_id: self.tenant_id.clone(),
            issuer: self.issuer.clone(),
            source: self.source.clone(),
            ceiling: self.ceiling,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a grant window in status planned.
pub fn new_grant_window(code: &str, title: &str) -> GrantWindow {
    GrantWindow {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        attributes: BTreeMap::new(),
        minimum: 0.0,
        tenant_id: "".to_string(),
        issuer: "".to_string(),
        source: "".to_string(),
        ceiling: 0.0,
        principal: "".to_string(),
        status: GrantWindowStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_grant_window("code-680", "title-353");
        assert_eq!(item.status, GrantWindowStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_grant_window("code-680", "title-353");
        if item.move_to(GrantWindowStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantWindowStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_grant_window("code-680", "title-353");
        item.status = GrantWindowStatus::Cancelled;
        assert!(
            item.move_to(GrantWindowStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
