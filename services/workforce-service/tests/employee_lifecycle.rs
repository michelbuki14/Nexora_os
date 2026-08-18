use nexora_workforce_service::models::{
    CreateEmployeeRequest, UpdateEmployeeStatusRequest, UpdateEmploymentRequest,
    CreateCompensationRequest, CreateLegalEntityRequest, CreateLocationRequest,
    CreateDepartmentRequest, CreateTeamRequest, CreatePositionRequest,
    CompensationFrequency, EmploymentType, EmployeeStatus, DocType,
};
use nexora_common::s3::document_object_key;
use nexora_common::audit::serialize_canonical;
use nexora_common::ulid::Ulid;
use nexora_common::money::{Money, CurrencyCode, RoundingPolicy};

#[test]
fn test_create_employee_request_deserialization() {
    let json = serde_json::json!({
        "employee_number": "EMP-001",
        "legal_name": "John Doe",
        "preferred_name": "John",
        "email": "john@company.com",
        "phone": "+1234567890",
        "date_of_birth": "1990-01-01",
        "gender": "male",
        "national_id": "123456789",
        "hire_date": "2024-01-15",
        "department_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "position_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "location_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "manager_employee_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "employment_type": "permanent"
    });

    let req: nexora_workforce_service::models::CreateEmployeeRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.employee_number, "EMP-001");
    assert_eq!(req.legal_name, "John Doe");
    assert_eq!(req.preferred_name, Some("John".to_string()));
    assert_eq!(req.email, "john@company.com");
}

#[test]
fn test_update_employee_status_request() {
    let json = serde_json::json!({
        "status": "terminated",
        "termination_date": "2024-12-31",
        "termination_reason": "Resigned"
    });

    let req: nexora_workforce_service::models::UpdateEmployeeStatusRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.status, EmployeeStatus::Terminated);
    assert_eq!(req.termination_date, Some(chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()));
    assert_eq!(req.termination_reason, Some("Resigned".to_string()));
}

#[test]
fn test_update_employment_request() {
    let json = serde_json::json!({
        "position_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "department_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "location_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "employment_type": "contract",
        "effective_date": "2024-06-01",
        "change_reason": "Promotion"
    });

    let req: nexora_workforce_service::models::UpdateEmploymentRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.position_ulid, Some("01ABCDEFGHIJKLMNOPQRSTUV".to_string()));
    assert_eq!(req.employment_type, Some(EmploymentType::Contract));
    assert_eq!(req.effective_date, chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap());
}

#[test]
fn test_create_compensation_request() {
    let json = serde_json::json!({
        "gross_amount_minor": 5000000,
        "currency_code": "CDF",
        "frequency": "monthly",
        "effective_date": "2024-01-01",
        "change_reason": "Annual review"
    });

    let req: nexora_workforce_service::models::CreateCompensationRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.gross_amount_minor, 5000000);
    assert_eq!(req.currency_code, "CDF");
    assert_eq!(req.frequency, CompensationFrequency::Monthly);
    assert_eq!(req.effective_date, chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
}

#[test]
fn test_legal_entity_request() {
    let json = serde_json::json!({
        "name": "Acme Corporation",
        "registration_number": "REG-12345",
        "country_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "address": { "street": "123 Main St", "city": "Kinshasa", "country": "CD" }
    });

    let req: nexora_workforce_service::models::CreateLegalEntityRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.name, "Acme Corporation");
    assert_eq!(req.registration_number, Some("REG-12345".to_string()));
}

#[test]
fn test_location_request() {
    let json = serde_json::json!({
        "legal_entity_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "name": "Kinshasa HQ",
        "code": "KIN-HQ",
        "address": { "street": "123 Main St", "city": "Kinshasa" },
        "timezone": "Africa/Lubumbashi"
    });

    let req: nexora_workforce_service::models::CreateLocationRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.name, "Kinshasa HQ");
    assert_eq!(req.code, Some("KIN-HQ".to_string()));
    assert_eq!(req.timezone, Some("Africa/Lubumbashi".to_string()));
}

#[test]
fn test_department_request() {
    let json = serde_json::json!({
        "name": "Engineering",
        "code": "ENG",
        "parent_department_ulid": null,
        "location_ulid": "01ABCDEFGHIJKLMNOPQRSTUV"
    });

    let req: nexora_workforce_service::models::CreateDepartmentRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.name, "Engineering");
    assert_eq!(req.code, Some("ENG".to_string()));
}

#[test]
fn test_team_request() {
    let json = serde_json::json!({
        "department_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "name": "Backend Team",
        "code": "BE"
    });

    let req: nexora_workforce_service::models::CreateTeamRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.name, "Backend Team");
    assert_eq!(req.code, Some("BE".to_string()));
}

#[test]
fn test_position_request() {
    let json = serde_json::json!({
        "title": "Senior Software Engineer",
        "code": "SSE",
        "department_ulid": "01ABCDEFGHIJKLMNOPQRSTUV",
        "job_grade": "E5",
        "description": "Senior backend engineer",
        "responsibilities": "Build APIs",
        "employment_type": "permanent"
    });

    let req: nexora_workforce_service::models::CreatePositionRequest =
        serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(req.title, "Senior Software Engineer");
    assert_eq!(req.code, Some("SSE".to_string()));
    assert_eq!(req.job_grade, Some("E5".to_string()));
    assert_eq!(req.employment_type, Some(EmploymentType::Permanent));
}

#[test]
fn test_compensation_frequency_enum() {
    assert_eq!(CompensationFrequency::Monthly.as_str(), "monthly");
    assert_eq!(CompensationFrequency::Annual.as_str(), "annual");
    assert_eq!(CompensationFrequency::Hourly.as_str(), "hourly");
    assert_eq!(CompensationFrequency::Weekly.as_str(), "weekly");
}

#[test]
fn test_employment_type_enum() {
    assert_eq!(EmploymentType::Permanent.as_str(), "permanent");
    assert_eq!(EmploymentType::Contract.as_str(), "contract");
    assert_eq!(EmploymentType::Intern.as_str(), "intern");
    assert_eq!(EmploymentType::Consultant.as_str(), "consultant");
}

#[test]
fn test_employee_status_enum() {
    assert_eq!(EmployeeStatus::Onboarding.as_str(), "onboarding");
    assert_eq!(EmployeeStatus::Active.as_str(), "active");
    assert_eq!(EmployeeStatus::OnLeave.as_str(), "on_leave");
    assert_eq!(EmployeeStatus::Terminated.as_str(), "terminated");
}

#[test]
fn test_doc_type_enum() {
    assert_eq!(DocType::Contract.as_str(), "contract");
    assert_eq!(DocType::IdDocument.as_str(), "id_document");
    assert_eq!(DocType::Certificate.as_str(), "certificate");
    assert_eq!(DocType::OfferLetter.as_str(), "offer_letter");
    assert_eq!(DocType::Payslip.as_str(), "payslip");
    assert_eq!(DocType::Other.as_str(), "other");
}

#[test]
fn test_s3_object_key_namespacing() {
    let key = document_object_key("TENANT123", "EMP456", "DOC789");
    assert_eq!(key, "TENANT123/EMP456/DOC789");

    let key2 = document_object_key("OTHER_TENANT", "EMP456", "DOC789");
    assert_ne!(key, key2);
}

#[test]
fn test_serialize_canonical_deterministic() {
    let value = serde_json::json!({
        "b": 2,
        "a": 1,
        "c": { "x": 1, "y": 2 }
    });

    let a = serialize_canonical(&value).unwrap();
    let b = serialize_canonical(&value).unwrap();
    assert_eq!(a, b);

    // Keys should be sorted
    let parsed: serde_json::Value = serde_json::from_str(&a).unwrap();
    let keys: Vec<&str> = parsed.as_object().unwrap().keys().map(|s| s.as_str()).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted);
}

#[test]
fn test_money_creation() {
    let m = nexora_common::money::Money::from_str_amount("125000.50", nexora_common::money::CurrencyCode::cdf());
    assert!(m.is_ok());
    let m = m.unwrap();
    assert_eq!(m.amount().to_string(), "125000.50");
    assert_eq!(m.currency().as_str(), "CDF");
}

#[test]
fn test_money_arithmetic() {
    let a = nexora_common::money::Money::from_str_amount("100.00", nexora_common::money::CurrencyCode::cdf()).unwrap();
    let b = nexora_common::money::Money::from_str_amount("50.00", nexora_common::money::CurrencyCode::cdf()).unwrap();
    let sum = a.checked_add(&b).unwrap();
    assert_eq!(sum.amount().to_string(), "150.00");
    assert_eq!(sum.currency().as_str(), "CDF");

    let diff = a.checked_sub(&b).unwrap();
    assert_eq!(diff.amount().to_string(), "50.00");
}

#[test]
fn test_money_currency_mismatch() {
    let a = nexora_common::money::Money::from_str_amount("100.00", nexora_common::money::CurrencyCode::cdf()).unwrap();
    let b = nexora_common::money::Money::from_str_amount("50.00", nexora_common::money::CurrencyCode::new("USD").unwrap()).unwrap();
    assert!(a.checked_add(&b).is_err());
}

#[test]
fn test_money_rounding() {
    let m = nexora_common::money::Money::from_str_amount("125000.00005", nexora_common::money::CurrencyCode::cdf()).unwrap();
    let rounded = m.rounded(nexora_common::money::RoundingPolicy::RoundHalfUp);
    assert_eq!(rounded.amount().to_string(), "125000.0001");
}

#[test]
fn test_ulid_generation() {
    let u1 = nexora_common::ulid::new_ulid();
    let u2 = nexora_common::ulid::new_ulid();
    assert_ne!(u1.to_string(), u2.to_string());
    assert_eq!(u1.to_string().len(), 26);
}