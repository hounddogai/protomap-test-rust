use super::*;

use acme_kit::text;

/// Where a lenient freeze tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientFreezeTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientFreezeTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientFreezeTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient freeze tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeTier {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub burst: i64,
    pub score: f64,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub category: String,
    pub tags: Vec<String>,
    pub status: LenientFreezeTierStatus,
}

impl LenientFreezeTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientFreezeTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientFreezeTierStatus) -> bool {
        match self.status {
            LenientFreezeTierStatus::Proposed => {
                match next {
                    LenientFreezeTierStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientFreezeTierStatus::Approved => {
                return next == LenientFreezeTierStatus::Rejected;
            }
            LenientFreezeTierStatus::Rejected => {
                match next {
                    LenientFreezeTierStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LenientFreezeTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient freeze tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.burst as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientFreezeTier {
        LenientFreezeTier {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            burst: self.burst,
            score: self.score,
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            category: self.category.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient freeze tier in status proposed.
pub fn new_lenient_freeze_tier(code: &str, title: &str) -> LenientFreezeTier {
    LenientFreezeTier {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        burst: 0,
        score: 0.0,
        ttl_seconds: 0,
        quantity: 0,
        category: "".to_string(),
        tags: Vec::new(),
        status: LenientFreezeTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_freeze_tier("code-122", "title-976");
        assert_eq!(item.status, LenientFreezeTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_lenient_freeze_tier("code-122", "title-976");
        if item.move_to(LenientFreezeTierStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientFreezeTierStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_freeze_tier("code-122", "title-976");
        item.status = LenientFreezeTierStatus::Withdrawn;
        assert!(
            item.move_to(LenientFreezeTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
