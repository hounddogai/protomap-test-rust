use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient forecast baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientForecastBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientForecastBaselineStatus {
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

impl std::fmt::Display for LenientForecastBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient forecast baseline as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastBaseline {
    pub code: String,
    pub title: String,
    pub notes: String,
    pub sequence: i64,
    pub attributes: BTreeMap<String, String>,
    pub created_at: i64,
    pub priority: i64,
    pub ratio: f64,
    pub capacity: i64,
    pub audience: String,
    pub duration_minutes: i64,
    pub status: LenientForecastBaselineStatus,
}

impl LenientForecastBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientForecastBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientForecastBaselineStatus) -> bool {
        match self.status {
            LenientForecastBaselineStatus::Planned => {
                match next {
                    LenientForecastBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientForecastBaselineStatus::Scheduled => {
                match next {
                    LenientForecastBaselineStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientForecastBaselineStatus::InProgress => {
                match next {
                    LenientForecastBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientForecastBaselineStatus::Completed => {
                match next {
                    LenientForecastBaselineStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientForecastBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient forecast baseline {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
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

    pub fn projected_value(&self) -> f64 {
        ((self.ratio + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientForecastBaseline {
        LenientForecastBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            notes: self.notes.clone(),
            sequence: self.sequence,
            attributes: self.attributes.clone(),
            created_at: self.created_at,
            priority: self.priority,
            ratio: self.ratio,
            capacity: self.capacity,
            audience: self.audience.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient forecast baseline in status planned.
pub fn new_lenient_forecast_baseline(code: &str, title: &str) -> LenientForecastBaseline {
    LenientForecastBaseline {
        code: code.to_string(),
        title: title.to_string(),
        notes: "".to_string(),
        sequence: 0,
        attributes: BTreeMap::new(),
        created_at: 0,
        priority: 0,
        ratio: 0.0,
        capacity: 0,
        audience: "".to_string(),
        duration_minutes: 0,
        status: LenientForecastBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_forecast_baseline("code-669", "title-776");
        assert_eq!(item.status, LenientForecastBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_lenient_forecast_baseline("code-669", "title-776");
        if item.move_to(LenientForecastBaselineStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientForecastBaselineStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_forecast_baseline("code-669", "title-776");
        item.status = LenientForecastBaselineStatus::Cancelled;
        assert!(
            item.move_to(LenientForecastBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
