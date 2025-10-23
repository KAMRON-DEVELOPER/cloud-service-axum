use crate::features::{
    listings::models::{ApartmentCondition, SaleType},
    users::schemas::UserOut,
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// -- =====================
// -- IN
// -- =====================
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct PictureIn {
    pub url: String,
    pub is_primary: Option<bool>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AddressIn {
    pub street_address: String,
    pub city: String,
    pub state_or_region: String,
    pub county_or_district: String,
    pub postal_code: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ApartmentIn {
    pub title: String,
    pub description: Option<String>,
    pub rooms: i32,
    pub beds: i32,
    pub baths: i32,
    pub address: AddressIn,
    pub amenities: Vec<String>,
    pub pictures: Vec<PictureIn>,
    pub area: f64,
    pub apartment_floor: i32,
    pub total_building_floors: i32,
    pub condition: ApartmentCondition,
    pub sale_type: SaleType,
    pub requirements: Option<String>,
    pub furnished: bool,
    pub pets_allowed: bool,
    pub has_elevator: bool,
    pub has_garden: bool,
    pub has_parking: bool,
    pub has_balcony: bool,
    pub has_ac: bool,
    pub has_heating: bool,
    pub distance_to_kindergarten: i32,
    pub distance_to_school: i32,
    pub distance_to_hospital: i32,
    pub distance_to_metro: i32,
    pub distance_to_bus_stop: i32,
    pub distance_to_shopping: i32,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ListingIn {
    pub price: BigDecimal,
    pub apartment: ApartmentIn,
    pub tags: Vec<String>,
}

// -- =====================
// -- OUT
// -- =====================
#[derive(Serialize, Deserialize)]
pub struct AmenityOut {
    pub id: Uuid,
    pub apartment_id: Uuid,
    pub amenity: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct PictureOut {
    pub id: Option<Uuid>,
    pub apartment_id: Option<Uuid>,
    pub url: Option<String>,
    pub is_primary: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct ApartmentOut {
    pub id: Option<Uuid>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub rooms: Option<i32>,
    pub beds: Option<i32>,
    pub baths: Option<i32>,
    pub address: Option<AddressOut>,
    pub pictures: Vec<PictureOut>,
    pub amenities: Vec<AmenityOut>,
    pub area: Option<f64>,
    pub apartment_floor: Option<i32>,
    pub total_building_floors: Option<i32>,
    pub condition: Option<ApartmentCondition>,
    pub sale_type: Option<SaleType>,
    pub requirements: Option<String>,
    pub furnished: Option<bool>,
    pub pets_allowed: Option<bool>,
    pub has_elevator: Option<bool>,
    pub has_garden: Option<bool>,
    pub has_parking: Option<bool>,
    pub has_balcony: Option<bool>,
    pub has_ac: Option<bool>,
    pub has_heating: Option<bool>,
    pub distance_to_kindergarten: Option<i32>,
    pub distance_to_school: Option<i32>,
    pub distance_to_hospital: Option<i32>,
    pub distance_to_metro: Option<i32>,
    pub distance_to_bus_stop: Option<i32>,
    pub distance_to_shopping: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize)]
pub struct TagOut {
    pub id: Uuid,
    pub listing_id: Uuid,
    pub tag: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ListingOut {
    pub id: Uuid,
    pub owner: UserOut,
    pub apartment: ApartmentOut,
    pub price: BigDecimal,
    pub tags: Vec<TagOut>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct AddressOut {
    pub id: Option<Uuid>,
    pub apartment_id: Option<Uuid>,
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub state_or_region: Option<String>,
    pub county_or_district: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize)]
pub struct ListingResponse {
    pub listings: Vec<ListingOut>,
    pub total: i64,
}
