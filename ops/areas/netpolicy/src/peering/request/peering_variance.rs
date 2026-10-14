use super::*;

use acme_kit::text;

/// Where a peering variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PeeringVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PeeringVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringVariance {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub ends_at: i64,
    pub batch_size: i64,
    pub pinned: bool,
    pub score: f64,
    pub source: String,
    pub name: String,
    pub fingerprint: String,
    pub status: PeeringVarianceStatus,
}

impl PeeringVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringVarianceStatus) -> bool {
        match self.status {
            PeeringVarianceStatus::Proposed => {
                return next == PeeringVarianceStatus::Approved;
            }
            PeeringVarianceStatus::Approved => {
                match next {
                    PeeringVarianceStatus::Rejected | PeeringVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringVarianceStatus::Rejected => {
                match next {
                    PeeringVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PeeringVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering variance {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ends_at as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringVariance {
        PeeringVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            ends_at: self.ends_at,
            batch_size: self.batch_size,
            pinned: self.pinned,
            score: self.score,
            source: self.source.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a peering variance in status proposed.
pub fn new_peering_variance(code: &str, label: &str) -> PeeringVariance {
    PeeringVariance {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        ends_at: 0,
        batch_size: 0,
        pinned: false,
        score: 0.0,
        source: "".to_string(),
        name: "".to_string(),
        fingerprint: "".to_string(),
        status: PeeringVarianceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_peering_variance("code-906", "label-233");
        assert_eq!(item.status, PeeringVarianceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_peering_variance("code-906", "label-233");
        if item.move_to(PeeringVarianceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringVarianceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_peering_variance("code-906", "label-233");
        item.status = PeeringVarianceStatus::Withdrawn;
        assert!(
            item.move_to(PeeringVarianceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
