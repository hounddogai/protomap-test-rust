use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves legacy dashboard agreements through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LegacyDashboardAgreementDesk {
    pub store: Rc<dyn LegacyDashboardAgreementStore>,
    pub validator: LegacyDashboardAgreementValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl LegacyDashboardAgreementDesk {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy dashboard agreement {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LegacyDashboardAgreementStatus::Granted)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is granted", code));
        Ok(())
    }

    pub fn retire(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("legacy dashboard agreement {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LegacyDashboardAgreementStatus::Expired)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is expired: {}", code, reason));
        Ok(())
    }

    pub fn sweep(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_legacy_dashboard_agreement_desk(
    store: Rc<dyn LegacyDashboardAgreementStore>,
    clock: Rc<dyn clock::Clock>,
) -> LegacyDashboardAgreementDesk {
    LegacyDashboardAgreementDesk {
        store: store.clone(),
        validator: default_legacy_dashboard_agreement_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
