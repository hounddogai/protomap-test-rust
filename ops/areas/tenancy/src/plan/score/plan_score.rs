use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a plan score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanScoreStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PlanScoreStatus {
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

impl std::fmt::Display for PlanScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanScore {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub ratio: f64,
    pub maximum: f64,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub active: bool,
    pub namespace: String,
    pub status: PlanScoreStatus,
}

impl PlanScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanScoreStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanScoreStatus) -> bool {
        match self.status {
            PlanScoreStatus::Draft => {
                return next == PlanScoreStatus::Pending || next == PlanScoreStatus::Active;
            }
            PlanScoreStatus::Pending => {
                return next == PlanScoreStatus::Active || next == PlanScoreStatus::Closed;
            }
            PlanScoreStatus::Active => {
                match next {
                    PlanScoreStatus::Suspended | PlanScoreStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanScoreStatus::Suspended => {
                return next == PlanScoreStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan score", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn net_load(&self) -> f64 {
        ((self.ratio - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanScore {
        PlanScore {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            ratio: self.ratio,
            maximum: self.maximum,
            tags: self.tags.clone(),
            attributes: self.attributes.clone(),
            active: self.active,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a plan score in status draft.
pub fn new_plan_score(code: &str, name: &str) -> PlanScore {
    PlanScore {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        ratio: 0.0,
        maximum: 0.0,
        tags: Vec::new(),
        attributes: BTreeMap::new(),
        active: false,
        namespace: "".to_string(),
        status: PlanScoreStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_plan_score("code-407", "name-896");
        assert_eq!(item.status, PlanScoreStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_plan_score("code-407", "name-896");
        if item.move_to(PlanScoreStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanScoreStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_plan_score("code-407", "name-896");
        item.status = PlanScoreStatus::Closed;
        assert!(item.move_to(PlanScoreStatus::Draft).is_err(), "expected move to fail");
    }
}
