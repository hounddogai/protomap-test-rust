use super::*;

use acme_kit::text;

/// Where a weighted plan window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPlanWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedPlanWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedPlanWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted plan window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPlanWindow {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub phone: String,
    pub ceiling: f64,
    pub sequence: i64,
    pub version: i64,
    pub notes: String,
    pub status: WeightedPlanWindowStatus,
}

impl WeightedPlanWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPlanWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPlanWindowStatus) -> bool {
        match self.status {
            WeightedPlanWindowStatus::Requested => {
                return next == WeightedPlanWindowStatus::Expired;
            }
            WeightedPlanWindowStatus::Granted => {
                match next {
                    WeightedPlanWindowStatus::Revoked | WeightedPlanWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPlanWindowStatus::Revoked => {
                match next {
                    WeightedPlanWindowStatus::Expired => {
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

    pub fn move_to(&mut self, next: WeightedPlanWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted plan window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted plan window", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.sequence as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPlanWindow {
        WeightedPlanWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            ceiling: self.ceiling,
            sequence: self.sequence,
            version: self.version,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a weighted plan window in status requested.
pub fn new_weighted_plan_window(code: &str, title: &str) -> WeightedPlanWindow {
    WeightedPlanWindow {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        phone: "".to_string(),
        ceiling: 0.0,
        sequence: 0,
        version: 0,
        notes: "".to_string(),
        status: WeightedPlanWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_plan_window("code-295", "title-336");
        assert_eq!(item.status, WeightedPlanWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_weighted_plan_window("code-295", "title-336");
        if item.move_to(WeightedPlanWindowStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPlanWindowStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_plan_window("code-295", "title-336");
        item.status = WeightedPlanWindowStatus::Expired;
        assert!(
            item.move_to(WeightedPlanWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
