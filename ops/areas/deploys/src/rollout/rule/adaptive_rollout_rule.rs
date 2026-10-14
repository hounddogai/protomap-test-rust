use super::*;

use acme_kit::text;

/// Where a adaptive rollout rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRolloutRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveRolloutRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveRolloutRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive rollout rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutRule {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub floor_value: f64,
    pub country: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub ratio: f64,
    pub status: AdaptiveRolloutRuleStatus,
}

impl AdaptiveRolloutRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRolloutRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRolloutRuleStatus) -> bool {
        match self.status {
            AdaptiveRolloutRuleStatus::Proposed => {
                match next {
                    AdaptiveRolloutRuleStatus::Rejected | AdaptiveRolloutRuleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRolloutRuleStatus::Approved => {
                return next == AdaptiveRolloutRuleStatus::Withdrawn;
            }
            AdaptiveRolloutRuleStatus::Rejected => {
                return next == AdaptiveRolloutRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRolloutRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollout rule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive rollout rule", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ratio + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRolloutRule {
        AdaptiveRolloutRule {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            floor_value: self.floor_value,
            country: self.country.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive rollout rule in status proposed.
pub fn new_adaptive_rollout_rule(code: &str, label: &str) -> AdaptiveRolloutRule {
    AdaptiveRolloutRule {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        floor_value: 0.0,
        country: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        ratio: 0.0,
        status: AdaptiveRolloutRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_rollout_rule("code-404", "label-335");
        assert_eq!(item.status, AdaptiveRolloutRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_adaptive_rollout_rule("code-404", "label-335");
        if item.move_to(AdaptiveRolloutRuleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRolloutRuleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_rollout_rule("code-404", "label-335");
        item.status = AdaptiveRolloutRuleStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveRolloutRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
