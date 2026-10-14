use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending timeline policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTimelinePolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingTimelinePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingTimelinePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending timeline policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelinePolicy {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub discount: f64,
    pub ratio: f64,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub margin: f64,
    pub sequence: i64,
    pub name: String,
    pub approved: bool,
    pub status: PendingTimelinePolicyStatus,
}

impl PendingTimelinePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTimelinePolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTimelinePolicyStatus) -> bool {
        match self.status {
            PendingTimelinePolicyStatus::Created => {
                match next {
                    PendingTimelinePolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTimelinePolicyStatus::Validated => {
                return next == PendingTimelinePolicyStatus::Published;
            }
            PendingTimelinePolicyStatus::Published => {
                match next {
                    PendingTimelinePolicyStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingTimelinePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending timeline policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending timeline policy", self.code)
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

    pub fn weighted_volume(&self) -> f64 {
        (self.ratio * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTimelinePolicy {
        PendingTimelinePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            discount: self.discount,
            ratio: self.ratio,
            tags: self.tags.clone(),
            attributes: self.attributes.clone(),
            margin: self.margin,
            sequence: self.sequence,
            name: self.name.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending timeline policy in status created.
pub fn new_pending_timeline_policy(code: &str, label: &str) -> PendingTimelinePolicy {
    PendingTimelinePolicy {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        discount: 0.0,
        ratio: 0.0,
        tags: Vec::new(),
        attributes: BTreeMap::new(),
        margin: 0.0,
        sequence: 0,
        name: "".to_string(),
        approved: false,
        status: PendingTimelinePolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_timeline_policy("code-261", "label-483");
        assert_eq!(item.status, PendingTimelinePolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_pending_timeline_policy("code-261", "label-483");
        if item.move_to(PendingTimelinePolicyStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTimelinePolicyStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_timeline_policy("code-261", "label-483");
        item.status = PendingTimelinePolicyStatus::Retired;
        assert!(
            item.move_to(PendingTimelinePolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
