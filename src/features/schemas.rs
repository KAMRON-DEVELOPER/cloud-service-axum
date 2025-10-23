// use crate::{
//     features::listings::models::{ApartmentCondition, SaleType},
//     utilities::errors::AppError,
// };
// use serde::{Deserialize, Serialize};
// use serde_with::{DisplayFromStr, serde_as};

// #[derive(Serialize, Deserialize, PartialEq, Eq, Default, Debug)]
// #[serde(rename_all = "camelCase")]
// pub enum Sort {
//     #[default]
//     Newest,
//     Cheap,
//     Expensive,
// }

// #[serde_as]
// #[derive(Deserialize, Serialize, Default, Debug)]
// #[serde(default, rename_all = "camelCase")]
// pub struct SearchParams {
//     // Basic filters
//     pub q: Option<String>,
//     pub sort: Option<Sort>,
//     pub country: String,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub max_price: Option<u64>,

//     // Property details
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub min_rooms: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub min_beds: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub min_baths: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub min_area: Option<i64>,

//     // Floor handling - clarified
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub apartment_floor: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub min_building_floors: Option<i64>,

//     pub condition: Option<ApartmentCondition>,
//     pub sale_type: Option<SaleType>,

//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub furnished: Option<bool>,
//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub pets_allowed: Option<bool>,
//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub has_elevator: Option<bool>,
//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub has_garden: Option<bool>,
//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub has_parking: Option<bool>,
//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub has_balcony: Option<bool>,
//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub has_ac: Option<bool>,
//     #[serde(deserialize_with = "deserialize_bool_from_any")]
//     pub has_heating: Option<bool>,

//     // Distances (in km)
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub max_distance_to_kindergarten: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub max_distance_to_school: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub max_distance_to_hospital: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub max_distance_to_metro: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub max_distance_to_bus_stop: Option<i64>,
//     #[serde_as(as = "Option<DisplayFromStr>")]
//     pub max_distance_to_shopping: Option<i64>,
// }

// fn deserialize_bool_from_any<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
// where
//     D: serde::Deserializer<'de>,
// {
//     Ok(Some(match String::deserialize(deserializer)?.as_str() {
//         "true" | "1" => true,
//         "false" | "0" => false,
//         _ => return Err(serde::de::Error::custom("expected true/false or 1/0")),
//     }))
// }

// // impl SearchParams {
// //     pub fn validate(&self) -> Result<(), AppError> {
// //         // Price validation
// //         if let (Some(min), Some(max)) = (self.min_price, self.max_price) {
// //             if min > max {
// //                 return Err(AppError::ValidationError(
// //                     "Minimum price cannot exceed maximum price".to_string(),
// //                 ));
// //             }
// //         }

// //         // Rooms validation
// //         if let (Some(min), Some(max)) = (self.min_rooms, self.max_rooms) {
// //             if min > max {
// //                 return Err(AppError::ValidationError(
// //                     "Minimum rooms cannot exceed maximum rooms".to_string(),
// //                 ));
// //             }
// //         }

// //         // Area validation
// //         if let (Some(min), Some(max)) = (self.min_area, self.max_area) {
// //             if min > max {
// //                 return Err(AppError::ValidationError(
// //                     "Minimum area cannot exceed maximum area".to_string(),
// //                 ));
// //             }
// //         }

// //         // Floor validation
// //         if let (Some(min), Some(max)) = (self.min_apartment_floor, self.max_apartment_floor) {
// //             if min > max {
// //                 return Err(AppError::ValidationError(
// //                     "Minimum floor cannot exceed maximum floor".to_string(),
// //                 ));
// //             }
// //         }

// //         Ok(())
// //     }
// // }

// #[derive(Deserialize, Serialize, Debug)]
// pub struct Pagination {
//     #[serde(default = "default_offset")]
//     pub offset: i64,
//     #[serde(default = "default_limit")]
//     pub limit: i64,
// }

// fn default_offset() -> i64 {
//     0
// }

// fn default_limit() -> i64 {
//     20
// }

// impl Pagination {
//     pub fn validate(&self) -> Result<(), AppError> {
//         if self.offset < 0 {
//             return Err(AppError::ValidationError(
//                 "Offset must be positive".to_string(),
//             ));
//         }

//         if self.limit < 0 {
//             return Err(AppError::ValidationError("Limit must positive".to_string()));
//         } else if self.limit == 0 {
//             return Err(AppError::ValidationError(
//                 "Limit must not be zero!".to_string(),
//             ));
//         }

//         if self.limit > 100 {
//             return Err(AppError::ValidationError(
//                 "Limit cannot exceed 100".to_string(),
//             ));
//         }

//         Ok(())
//     }
// }

// #[derive(Deserialize, Debug)]
// pub struct ListingQuery {
//     #[serde(flatten)]
//     pub pagination: Pagination,
//     #[serde(flatten)]
//     pub search_params: SearchParams,
// }
