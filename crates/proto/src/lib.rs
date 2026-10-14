//! Protobuf messages and tonic service bindings, compiled from the vendored API contracts.

pub mod acme {
    pub mod accounting {
        pub mod v1 {
            tonic::include_proto!("acme.accounting.v1");
        }
    }
    pub mod admissions {
        pub mod v1 {
            tonic::include_proto!("acme.admissions.v1");
        }
    }
    pub mod analytics {
        pub mod v1 {
            tonic::include_proto!("acme.analytics.v1");
        }
    }
    pub mod anomalies {
        pub mod v1 {
            tonic::include_proto!("acme.anomalies.v1");
        }
    }
    pub mod appointments {
        pub mod v1 {
            tonic::include_proto!("acme.appointments.v1");
        }
    }
    pub mod assets {
        pub mod v1 {
            tonic::include_proto!("acme.assets.v1");
        }
    }
    pub mod audit {
        pub mod v1 {
            tonic::include_proto!("acme.audit.v1");
        }
    }
    pub mod authorization {
        pub mod v1 {
            tonic::include_proto!("acme.authorization.v1");
        }
    }
    pub mod boxoffice {
        pub mod v1 {
            tonic::include_proto!("acme.boxoffice.v1");
        }
    }
    pub mod broadcasts {
        pub mod v1 {
            tonic::include_proto!("acme.broadcasts.v1");
        }
    }
    pub mod careplans {
        pub mod v1 {
            tonic::include_proto!("acme.careplans.v1");
        }
    }
    pub mod catalog {
        pub mod v1 {
            tonic::include_proto!("acme.catalog.v1");
        }
    }
    pub mod certificates {
        pub mod v1 {
            tonic::include_proto!("acme.certificates.v1");
        }
    }
    pub mod chat {
        pub mod v1 {
            tonic::include_proto!("acme.chat.v1");
        }
    }
    pub mod checkout {
        pub mod v1 {
            tonic::include_proto!("acme.checkout.v1");
        }
    }
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("acme.common.v1");
        }
    }
    pub mod community {
        pub mod v1 {
            tonic::include_proto!("acme.community.v1");
        }
    }
    pub mod concessions {
        pub mod v1 {
            tonic::include_proto!("acme.concessions.v1");
        }
    }
    pub mod configuration {
        pub mod v1 {
            tonic::include_proto!("acme.configuration.v1");
        }
    }
    pub mod contracts {
        pub mod v1 {
            tonic::include_proto!("acme.contracts.v1");
        }
    }
    pub mod customers {
        pub mod v1 {
            tonic::include_proto!("acme.customers.v1");
        }
    }
    pub mod datasets {
        pub mod v1 {
            tonic::include_proto!("acme.datasets.v1");
        }
    }
    pub mod deliveries {
        pub mod v1 {
            tonic::include_proto!("acme.deliveries.v1");
        }
    }
    pub mod dispatch {
        pub mod v1 {
            tonic::include_proto!("acme.dispatch.v1");
        }
    }
    pub mod documents {
        pub mod v1 {
            tonic::include_proto!("acme.documents.v1");
        }
    }
    pub mod drivers {
        pub mod v1 {
            tonic::include_proto!("acme.drivers.v1");
        }
    }
    pub mod edgegateway {
        pub mod v1 {
            tonic::include_proto!("acme.edgegateway.v1");
        }
    }
    pub mod employees {
        pub mod v1 {
            tonic::include_proto!("acme.employees.v1");
        }
    }
    pub mod expenses {
        pub mod v1 {
            tonic::include_proto!("acme.expenses.v1");
        }
    }
    pub mod experiments {
        pub mod v1 {
            tonic::include_proto!("acme.experiments.v1");
        }
    }
    pub mod fanportal {
        pub mod v1 {
            tonic::include_proto!("acme.fanportal.v1");
        }
    }
    pub mod featureflags {
        pub mod v1 {
            tonic::include_proto!("acme.featureflags.v1");
        }
    }
    pub mod fleet {
        pub mod v1 {
            tonic::include_proto!("acme.fleet.v1");
        }
    }
    pub mod forms {
        pub mod v1 {
            tonic::include_proto!("acme.forms.v1");
        }
    }
    pub mod fuel {
        pub mod v1 {
            tonic::include_proto!("acme.fuel.v1");
        }
    }
    pub mod fulfillment {
        pub mod v1 {
            tonic::include_proto!("acme.fulfillment.v1");
        }
    }
    pub mod giftcards {
        pub mod v1 {
            tonic::include_proto!("acme.giftcards.v1");
        }
    }
    pub mod helpdesk {
        pub mod v1 {
            tonic::include_proto!("acme.helpdesk.v1");
        }
    }
    pub mod identity {
        pub mod v1 {
            tonic::include_proto!("acme.identity.v1");
        }
    }
    pub mod insurance {
        pub mod v1 {
            tonic::include_proto!("acme.insurance.v1");
        }
    }
    pub mod inventory {
        pub mod v1 {
            tonic::include_proto!("acme.inventory.v1");
        }
    }
    pub mod invoicing {
        pub mod v1 {
            tonic::include_proto!("acme.invoicing.v1");
        }
    }
    pub mod itsm {
        pub mod v1 {
            tonic::include_proto!("acme.itsm.v1");
        }
    }
    pub mod keymanagement {
        pub mod v1 {
            tonic::include_proto!("acme.keymanagement.v1");
        }
    }
    pub mod labeling {
        pub mod v1 {
            tonic::include_proto!("acme.labeling.v1");
        }
    }
    pub mod ledger {
        pub mod v1 {
            tonic::include_proto!("acme.ledger.v1");
        }
    }
    pub mod lending {
        pub mod v1 {
            tonic::include_proto!("acme.lending.v1");
        }
    }
    pub mod lineage {
        pub mod v1 {
            tonic::include_proto!("acme.lineage.v1");
        }
    }
    pub mod loyalty {
        pub mod v1 {
            tonic::include_proto!("acme.loyalty.v1");
        }
    }
    pub mod maintenance {
        pub mod v1 {
            tonic::include_proto!("acme.maintenance.v1");
        }
    }
    pub mod media {
        pub mod v1 {
            tonic::include_proto!("acme.media.v1");
        }
    }
    pub mod messaging {
        pub mod v1 {
            tonic::include_proto!("acme.messaging.v1");
        }
    }
    pub mod metrics {
        pub mod v1 {
            tonic::include_proto!("acme.metrics.v1");
        }
    }
    pub mod notifications {
        pub mod v1 {
            tonic::include_proto!("acme.notifications.v1");
        }
    }
    pub mod onboarding {
        pub mod v1 {
            tonic::include_proto!("acme.onboarding.v1");
        }
    }
    pub mod patientapp {
        pub mod v1 {
            tonic::include_proto!("acme.patientapp.v1");
        }
    }
    pub mod patients {
        pub mod v1 {
            tonic::include_proto!("acme.patients.v1");
        }
    }
    pub mod payroll {
        pub mod v1 {
            tonic::include_proto!("acme.payroll.v1");
        }
    }
    pub mod performers {
        pub mod v1 {
            tonic::include_proto!("acme.performers.v1");
        }
    }
    pub mod personalization {
        pub mod v1 {
            tonic::include_proto!("acme.personalization.v1");
        }
    }
    pub mod prescriptions {
        pub mod v1 {
            tonic::include_proto!("acme.prescriptions.v1");
        }
    }
    pub mod pricing {
        pub mod v1 {
            tonic::include_proto!("acme.pricing.v1");
        }
    }
    pub mod procurement {
        pub mod v1 {
            tonic::include_proto!("acme.procurement.v1");
        }
    }
    pub mod promotions {
        pub mod v1 {
            tonic::include_proto!("acme.promotions.v1");
        }
    }
    pub mod ratelimiting {
        pub mod v1 {
            tonic::include_proto!("acme.ratelimiting.v1");
        }
    }
    pub mod reviews {
        pub mod v1 {
            tonic::include_proto!("acme.reviews.v1");
        }
    }
    pub mod routing {
        pub mod v1 {
            tonic::include_proto!("acme.routing.v1");
        }
    }
    pub mod search {
        pub mod v1 {
            tonic::include_proto!("acme.search.v1");
        }
    }
    pub mod seating {
        pub mod v1 {
            tonic::include_proto!("acme.seating.v1");
        }
    }
    pub mod secrets {
        pub mod v1 {
            tonic::include_proto!("acme.secrets.v1");
        }
    }
    pub mod sellers {
        pub mod v1 {
            tonic::include_proto!("acme.sellers.v1");
        }
    }
    pub mod shows {
        pub mod v1 {
            tonic::include_proto!("acme.shows.v1");
        }
    }
    pub mod sponsorships {
        pub mod v1 {
            tonic::include_proto!("acme.sponsorships.v1");
        }
    }
    pub mod subscriptions {
        pub mod v1 {
            tonic::include_proto!("acme.subscriptions.v1");
        }
    }
    pub mod suppliers {
        pub mod v1 {
            tonic::include_proto!("acme.suppliers.v1");
        }
    }
    pub mod telehealth {
        pub mod v1 {
            tonic::include_proto!("acme.telehealth.v1");
        }
    }
    pub mod timekeeping {
        pub mod v1 {
            tonic::include_proto!("acme.timekeeping.v1");
        }
    }
    pub mod tours {
        pub mod v1 {
            tonic::include_proto!("acme.tours.v1");
        }
    }
    pub mod treasury {
        pub mod v1 {
            tonic::include_proto!("acme.treasury.v1");
        }
    }
    pub mod venues {
        pub mod v1 {
            tonic::include_proto!("acme.venues.v1");
        }
    }
    pub mod warehousing {
        pub mod v1 {
            tonic::include_proto!("acme.warehousing.v1");
        }
    }
    pub mod workplaces {
        pub mod v1 {
            tonic::include_proto!("acme.workplaces.v1");
        }
    }
    pub mod yards {
        pub mod v1 {
            tonic::include_proto!("acme.yards.v1");
        }
    }
}
