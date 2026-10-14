use super::*;

use acme_kit::text;

/// Where a weighted credential bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCredentialBundleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedCredentialBundleStatus {
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

impl std::fmt::Display for WeightedCredentialBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted credential bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundle {
    pub code: String,
    pub title: String,
    pub country: String,
    pub rate: f64,
    pub maximum: f64,
    pub status: WeightedCredentialBundleStatus,
}

impl WeightedCredentialBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCredentialBundleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCredentialBundleStatus) -> bool {
        match self.status {
            WeightedCredentialBundleStatus::Planned => {
                match next {
                    WeightedCredentialBundleStatus::InProgress | WeightedCredentialBundleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialBundleStatus::Scheduled => {
                return next == WeightedCredentialBundleStatus::Cancelled;
            }
            WeightedCredentialBundleStatus::InProgress => {
                match next {
                    WeightedCredentialBundleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialBundleStatus::Completed => {
                return next == WeightedCredentialBundleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedCredentialBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted credential bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted credential bundle", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.maximum * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedCredentialBundle {
        WeightedCredentialBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            rate: self.rate,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted credential bundle in status planned.
pub fn new_weighted_credential_bundle(code: &str, title: &str) -> WeightedCredentialBundle {
    WeightedCredentialBundle {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        rate: 0.0,
        maximum: 0.0,
        status: WeightedCredentialBundleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_credential_bundle("code-656", "title-115");
        assert_eq!(item.status, WeightedCredentialBundleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_weighted_credential_bundle("code-656", "title-115");
        if item.move_to(WeightedCredentialBundleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedCredentialBundleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_credential_bundle("code-656", "title-115");
        item.status = WeightedCredentialBundleStatus::Cancelled;
        assert!(
            item.move_to(WeightedCredentialBundleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
