use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted segment policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSegmentPolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl WeightedSegmentPolicyStatus {
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

impl std::fmt::Display for WeightedSegmentPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted segment policy as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentPolicy {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub ends_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub country: String,
    pub status: WeightedSegmentPolicyStatus,
}

impl WeightedSegmentPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSegmentPolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSegmentPolicyStatus) -> bool {
        match self.status {
            WeightedSegmentPolicyStatus::Draft => {
                match next {
                    WeightedSegmentPolicyStatus::Pending | WeightedSegmentPolicyStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentPolicyStatus::Pending => {
                return next == WeightedSegmentPolicyStatus::Active || next == WeightedSegmentPolicyStatus::Suspended;
            }
            WeightedSegmentPolicyStatus::Active => {
                match next {
                    WeightedSegmentPolicyStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentPolicyStatus::Suspended => {
                match next {
                    WeightedSegmentPolicyStatus::Closed => {
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

    pub fn move_to(&mut self, next: WeightedSegmentPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted segment policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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
        self.ends_at as f64
    }

    pub fn copy(&self) -> WeightedSegmentPolicy {
        WeightedSegmentPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            ends_at: self.ends_at,
            attributes: self.attributes.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted segment policy in status draft.
pub fn new_weighted_segment_policy(code: &str, label: &str) -> WeightedSegmentPolicy {
    WeightedSegmentPolicy {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        ends_at: 0,
        attributes: BTreeMap::new(),
        country: "".to_string(),
        status: WeightedSegmentPolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_weighted_segment_policy("code-284", "label-704");
        assert_eq!(item.status, WeightedSegmentPolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_weighted_segment_policy("code-284", "label-704");
        if item.move_to(WeightedSegmentPolicyStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedSegmentPolicyStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_weighted_segment_policy("code-284", "label-704");
        item.status = WeightedSegmentPolicyStatus::Closed;
        assert!(
            item.move_to(WeightedSegmentPolicyStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
