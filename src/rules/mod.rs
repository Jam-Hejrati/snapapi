//! Independently documented compatibility rules and their default classification.

use crate::finding::{Direction, Severity};

#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub id: &'static str,
    pub request: Severity,
    pub response: Severity,
    pub summary: &'static str,
    pub explanation: &'static str,
    pub documentation: &'static str,
}

impl Rule {
    pub fn severity(self, direction: Direction) -> Severity {
        match direction {
            Direction::Request => self.request,
            Direction::Response => self.response,
        }
    }
}

pub mod operation_removed;
pub use operation_removed::RULE as OPERATION_REMOVED;
pub mod operation_added;
pub use operation_added::RULE as OPERATION_ADDED;
pub mod operation_deprecated;
pub use operation_deprecated::RULE as OPERATION_DEPRECATED;
pub mod security_requirement_added;
pub use security_requirement_added::RULE as SECURITY_REQUIREMENT_ADDED;
pub mod security_requirement_changed;
pub use security_requirement_changed::RULE as SECURITY_REQUIREMENT_CHANGED;
pub mod security_requirement_removed;
pub use security_requirement_removed::RULE as SECURITY_REQUIREMENT_REMOVED;
pub mod parameter_removed;
pub use parameter_removed::RULE as PARAMETER_REMOVED;
pub mod required_parameter_added;
pub use required_parameter_added::RULE as REQUIRED_PARAMETER_ADDED;
pub mod optional_parameter_added;
pub use optional_parameter_added::RULE as OPTIONAL_PARAMETER_ADDED;
pub mod parameter_became_required;
pub use parameter_became_required::RULE as PARAMETER_BECAME_REQUIRED;
pub mod parameter_became_optional;
pub use parameter_became_optional::RULE as PARAMETER_BECAME_OPTIONAL;
pub mod parameter_location_changed;
pub use parameter_location_changed::RULE as PARAMETER_LOCATION_CHANGED;
pub mod request_body_became_required;
pub use request_body_became_required::RULE as REQUEST_BODY_BECAME_REQUIRED;
pub mod request_body_became_optional;
pub use request_body_became_optional::RULE as REQUEST_BODY_BECAME_OPTIONAL;
pub mod request_media_type_removed;
pub use request_media_type_removed::RULE as REQUEST_MEDIA_TYPE_REMOVED;
pub mod request_media_type_added;
pub use request_media_type_added::RULE as REQUEST_MEDIA_TYPE_ADDED;
pub mod response_status_removed;
pub use response_status_removed::RULE as RESPONSE_STATUS_REMOVED;
pub mod response_status_added;
pub use response_status_added::RULE as RESPONSE_STATUS_ADDED;
pub mod response_media_type_removed;
pub use response_media_type_removed::RULE as RESPONSE_MEDIA_TYPE_REMOVED;
pub mod response_media_type_added;
pub use response_media_type_added::RULE as RESPONSE_MEDIA_TYPE_ADDED;
pub mod schema_type_changed;
pub use schema_type_changed::RULE as SCHEMA_TYPE_CHANGED;
pub mod property_removed;
pub use property_removed::RULE as PROPERTY_REMOVED;
pub mod property_added;
pub use property_added::RULE as PROPERTY_ADDED;
pub mod property_became_required;
pub use property_became_required::RULE as PROPERTY_BECAME_REQUIRED;
pub mod property_became_optional;
pub use property_became_optional::RULE as PROPERTY_BECAME_OPTIONAL;
pub mod enum_value_removed;
pub use enum_value_removed::RULE as ENUM_VALUE_REMOVED;
pub mod enum_value_added;
pub use enum_value_added::RULE as ENUM_VALUE_ADDED;
pub mod became_nullable;
pub use became_nullable::RULE as BECAME_NULLABLE;
pub mod became_non_nullable;
pub use became_non_nullable::RULE as BECAME_NON_NULLABLE;
pub mod constraint_tightened;
pub use constraint_tightened::RULE as CONSTRAINT_TIGHTENED;
pub mod constraint_loosened;
pub use constraint_loosened::RULE as CONSTRAINT_LOOSENED;
pub mod composition_changed;
pub use composition_changed::RULE as COMPOSITION_CHANGED;
pub mod schema_changed;
pub use schema_changed::RULE as SCHEMA_CHANGED;
pub mod description_changed;
pub use description_changed::RULE as DESCRIPTION_CHANGED;
pub mod summary_changed;
pub use summary_changed::RULE as SUMMARY_CHANGED;
pub mod example_changed;
pub use example_changed::RULE as EXAMPLE_CHANGED;

pub const ALL: &[Rule] = &[
    OPERATION_REMOVED,
    OPERATION_ADDED,
    OPERATION_DEPRECATED,
    SECURITY_REQUIREMENT_ADDED,
    SECURITY_REQUIREMENT_CHANGED,
    SECURITY_REQUIREMENT_REMOVED,
    PARAMETER_REMOVED,
    REQUIRED_PARAMETER_ADDED,
    OPTIONAL_PARAMETER_ADDED,
    PARAMETER_BECAME_REQUIRED,
    PARAMETER_BECAME_OPTIONAL,
    PARAMETER_LOCATION_CHANGED,
    REQUEST_BODY_BECAME_REQUIRED,
    REQUEST_BODY_BECAME_OPTIONAL,
    REQUEST_MEDIA_TYPE_REMOVED,
    REQUEST_MEDIA_TYPE_ADDED,
    RESPONSE_STATUS_REMOVED,
    RESPONSE_STATUS_ADDED,
    RESPONSE_MEDIA_TYPE_REMOVED,
    RESPONSE_MEDIA_TYPE_ADDED,
    SCHEMA_TYPE_CHANGED,
    PROPERTY_REMOVED,
    PROPERTY_ADDED,
    PROPERTY_BECAME_REQUIRED,
    PROPERTY_BECAME_OPTIONAL,
    ENUM_VALUE_REMOVED,
    ENUM_VALUE_ADDED,
    BECAME_NULLABLE,
    BECAME_NON_NULLABLE,
    CONSTRAINT_TIGHTENED,
    CONSTRAINT_LOOSENED,
    COMPOSITION_CHANGED,
    SCHEMA_CHANGED,
    DESCRIPTION_CHANGED,
    SUMMARY_CHANGED,
    EXAMPLE_CHANGED,
];
