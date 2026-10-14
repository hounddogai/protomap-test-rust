use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a sink draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SinkDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SinkDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraft {
    pub code: String,
    pub name: String,
    pub description: String,
    pub weight: f64,
    pub minimum: f64,
    pub cost: f64,
    pub label: String,
    pub visible: bool,
    pub timezone: String,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub status: SinkDraftStatus,
}

impl SinkDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkDraftStatus) -> bool {
        match self.status {
            SinkDraftStatus::Created => {
                match next {
                    SinkDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkDraftStatus::Validated => {
                match next {
                    SinkDraftStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkDraftStatus::Published => {
                return next == SinkDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "sink draft", self.code)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.weight + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkDraft {
        SinkDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            weight: self.weight,
            minimum: self.minimum,
            cost: self.cost,
            label: self.label.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a sink draft in status created.
pub fn new_sink_draft(code: &str, name: &str) -> SinkDraft {
    SinkDraft {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        weight: 0.0,
        minimum: 0.0,
        cost: 0.0,
        label: "".to_string(),
        visible: false,
        timezone: "".to_string(),
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        status: SinkDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_sink_draft("code-519", "name-879");
        assert_eq!(item.status, SinkDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_sink_draft("code-519", "name-879");
        if item.move_to(SinkDraftStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkDraftStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_sink_draft("code-519", "name-879");
        item.status = SinkDraftStatus::Retired;
        assert!(item.move_to(SinkDraftStatus::Created).is_err(), "expected move to fail");
    }
}
