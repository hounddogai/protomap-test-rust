use super::*;

use acme_kit::text;

/// Where a egress baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EgressBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EgressBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One egress baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressBaseline {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub sequence: i64,
    pub locale: String,
    pub source: String,
    pub country: String,
    pub revision: i64,
    pub duration_minutes: i64,
    pub channel: String,
    pub status: EgressBaselineStatus,
}

impl EgressBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressBaselineStatus) -> bool {
        match self.status {
            EgressBaselineStatus::Created => {
                match next {
                    EgressBaselineStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EgressBaselineStatus::Validated => {
                return next == EgressBaselineStatus::Published || next == EgressBaselineStatus::Retired;
            }
            EgressBaselineStatus::Published => {
                match next {
                    EgressBaselineStatus::Retired => {
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

    pub fn move_to(&mut self, next: EgressBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressBaseline {
        EgressBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            sequence: self.sequence,
            locale: self.locale.clone(),
            source: self.source.clone(),
            country: self.country.clone(),
            revision: self.revision,
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a egress baseline in status created.
pub fn new_egress_baseline(code: &str, title: &str) -> EgressBaseline {
    EgressBaseline {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        sequence: 0,
        locale: "".to_string(),
        source: "".to_string(),
        country: "".to_string(),
        revision: 0,
        duration_minutes: 0,
        channel: "".to_string(),
        status: EgressBaselineStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_egress_baseline("code-493", "title-35");
        assert_eq!(item.status, EgressBaselineStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_egress_baseline("code-493", "title-35");
        if item.move_to(EgressBaselineStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressBaselineStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_egress_baseline("code-493", "title-35");
        item.status = EgressBaselineStatus::Retired;
        assert!(
            item.move_to(EgressBaselineStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
