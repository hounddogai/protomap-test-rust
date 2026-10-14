use super::*;

use acme_kit::text;

/// Where a slo score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SloScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SloScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloScore {
    pub code: String,
    pub name: String,
    pub country: String,
    pub issuer: String,
    pub revision: i64,
    pub rate: f64,
    pub title: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub source: String,
    pub status: SloScoreStatus,
}

impl SloScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloScoreStatus) -> bool {
        match self.status {
            SloScoreStatus::Proposed => {
                return next == SloScoreStatus::Approved || next == SloScoreStatus::Rejected;
            }
            SloScoreStatus::Approved => {
                match next {
                    SloScoreStatus::Rejected | SloScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloScoreStatus::Rejected => {
                match next {
                    SloScoreStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SloScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.revision as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloScore {
        SloScore {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            issuer: self.issuer.clone(),
            revision: self.revision,
            rate: self.rate,
            title: self.title.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a slo score in status proposed.
pub fn new_slo_score(code: &str, name: &str) -> SloScore {
    SloScore {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        issuer: "".to_string(),
        revision: 0,
        rate: 0.0,
        title: "".to_string(),
        tags: Vec::new(),
        pinned: false,
        source: "".to_string(),
        status: SloScoreStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_slo_score("code-490", "name-242");
        assert_eq!(item.status, SloScoreStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_slo_score("code-490", "name-242");
        if item.move_to(SloScoreStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloScoreStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_slo_score("code-490", "name-242");
        item.status = SloScoreStatus::Withdrawn;
        assert!(item.move_to(SloScoreStatus::Proposed).is_err(), "expected move to fail");
    }
}
