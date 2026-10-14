use super::*;

use acme_kit::text;

/// Where a silence estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SilenceEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SilenceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceEstimate {
    pub code: String,
    pub label: String,
    pub name: String,
    pub external_ref: String,
    pub scope: String,
    pub score: f64,
    pub locale: String,
    pub starts_at: i64,
    pub archived: bool,
    pub status: SilenceEstimateStatus,
}

impl SilenceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceEstimateStatus) -> bool {
        match self.status {
            SilenceEstimateStatus::Created => {
                match next {
                    SilenceEstimateStatus::Validated | SilenceEstimateStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceEstimateStatus::Validated => {
                return next == SilenceEstimateStatus::Published;
            }
            SilenceEstimateStatus::Published => {
                return next == SilenceEstimateStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.score + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceEstimate {
        SilenceEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            score: self.score,
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a silence estimate in status created.
pub fn new_silence_estimate(code: &str, label: &str) -> SilenceEstimate {
    SilenceEstimate {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        external_ref: "".to_string(),
        scope: "".to_string(),
        score: 0.0,
        locale: "".to_string(),
        starts_at: 0,
        archived: false,
        status: SilenceEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_silence_estimate("code-644", "label-767");
        assert_eq!(item.status, SilenceEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_silence_estimate("code-644", "label-767");
        if item.move_to(SilenceEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_silence_estimate("code-644", "label-767");
        item.status = SilenceEstimateStatus::Retired;
        assert!(
            item.move_to(SilenceEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
