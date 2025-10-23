use crate::features::listings::models::{ApartmentCondition, SaleType};
use crate::features::listings::schemas::{
    AddressOut, AmenityOut, ApartmentOut, ListingIn, ListingOut, PictureOut, TagOut,
};
use crate::features::schemas::{ListingQuery, Sort};
use crate::features::users::models::{UserRole, UserStatus};
use crate::features::users::schemas::UserOut;
use crate::services::ai::AI;
use crate::utilities::errors::AppError;
use axum::extract::Multipart;
use chrono::{DateTime, Utc};
use object_store::gcp::GoogleCloudStorage;
use object_store::{ObjectStore, path::Path as ObjectStorePath};

use qdrant_client::qdrant::{
    Condition, Filter, Fusion, NamedVectors, PointId, PointStruct, PrefetchQueryBuilder, Query,
    QueryPointsBuilder, UpsertPointsBuilder, Vector,
};
use qdrant_client::{Payload, Qdrant};
use serde_json::json;
use sqlx::QueryBuilder;
use sqlx::{FromRow, PgPool, types::BigDecimal, types::Json};
use tracing::{debug, warn};
use uuid::Uuid;

#[derive(FromRow)]
pub struct ListingJoined {
    pub listing_id: Uuid,
    pub price: BigDecimal,
    pub listing_created_at: DateTime<Utc>,
    pub listing_updated_at: DateTime<Utc>,

    // owner
    pub owner_id: Uuid,
    pub owner_full_name: String,
    pub owner_email: String,
    pub owner_phone: String,
    pub owner_picture: Option<String>,
    pub owner_role: UserRole,
    pub owner_status: UserStatus,
    pub owner_email_verified: bool,
    pub owner_oauth_user_id: Option<String>,
    pub owner_created_at: DateTime<Utc>,
    pub owner_updated_at: DateTime<Utc>,

    // apartment
    pub apartment_id: Option<Uuid>,
    pub apartment_title: Option<String>,
    pub apartment_description: Option<String>,
    pub apartment_rooms: Option<i32>,
    pub apartment_beds: Option<i32>,
    pub apartment_baths: Option<i32>,
    pub apartment_area: Option<f64>,
    pub apartment_floor: Option<i32>,
    pub apartment_total_building_floors: Option<i32>,
    pub apartment_condition: Option<ApartmentCondition>,
    pub apartment_sale_type: Option<SaleType>,
    pub apartment_requirements: Option<String>,
    pub apartment_furnished: Option<bool>,
    pub apartment_pets_allowed: Option<bool>,
    pub apartment_has_elevator: Option<bool>,
    pub apartment_has_garden: Option<bool>,
    pub apartment_has_parking: Option<bool>,
    pub apartment_has_balcony: Option<bool>,
    pub apartment_has_ac: Option<bool>,
    pub apartment_has_heating: Option<bool>,
    pub distance_to_kindergarten: Option<i32>,
    pub distance_to_school: Option<i32>,
    pub distance_to_hospital: Option<i32>,
    pub distance_to_metro: Option<i32>,
    pub distance_to_bus_stop: Option<i32>,
    pub distance_to_shopping: Option<i32>,

    pub apartment_created_at: Option<DateTime<Utc>>,
    pub apartment_updated_at: Option<DateTime<Utc>>,

    // address
    pub address_id: Option<Uuid>,
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub state_or_region: Option<String>,
    pub county_or_district: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub address_created_at: Option<DateTime<Utc>>,
    pub address_updated_at: Option<DateTime<Utc>>,

    // arrays
    pub tags: Option<Json<Vec<TagOut>>>,
    pub amenities: Option<Json<Vec<AmenityOut>>>,
    pub pictures: Option<Json<Vec<PictureOut>>>,
}

pub async fn get_many_listings(
    pool: &PgPool,
    listing_query: &ListingQuery,
    qdrant: Qdrant,
    ai: AI,
) -> Result<(Vec<ListingOut>, i64), AppError> {
    let ListingQuery {
        pagination,
        search_params,
    } = listing_query;
    // Perform vector search if query exists
    let vector_matched_ids = if let Some(q) = &search_params.q {
        if !q.trim().is_empty() {
            perform_hybrid_search(q.trim(), search_params.country.clone(), &qdrant, &ai).await?
        } else {
            None
        }
    } else {
        None
    };

    let select_base = r#"
    SELECT
        l.id AS listing_id,
        l.price,
        l.created_at AS listing_created_at,
        l.updated_at AS listing_updated_at,

        -- owner fields ...
        u.id AS owner_id,
        u.full_name AS owner_full_name,
        u.email AS owner_email,
        u.phone_number AS owner_phone,
        u.picture AS owner_picture,
        u.role AS owner_role,
        u.status AS owner_status,
        u.email_verified AS owner_email_verified,
        u.oauth_user_id AS owner_oauth_user_id,
        u.created_at AS owner_created_at,
        u.updated_at AS owner_updated_at,

        -- apartment fields ...
        a.id AS apartment_id,
        a.title AS apartment_title,
        a.description AS apartment_description,
        a.rooms AS apartment_rooms,
        a.beds AS apartment_beds,
        a.baths AS apartment_baths,
        a.area AS apartment_area,
        a.apartment_floor AS apartment_floor,
        a.total_building_floors AS apartment_total_building_floors,
        a.condition AS apartment_condition,
        a.sale_type AS apartment_sale_type,
        a.requirements AS apartment_requirements,
        a.furnished AS apartment_furnished,
        a.pets_allowed AS apartment_pets_allowed,
        a.has_elevator AS apartment_has_elevator,
        a.has_garden AS apartment_has_garden,
        a.has_parking AS apartment_has_parking,
        a.has_balcony AS apartment_has_balcony,
        a.has_ac AS apartment_has_ac,
        a.has_heating AS apartment_has_heating,
        a.distance_to_kindergarten,
        a.distance_to_school,
        a.distance_to_hospital,
        a.distance_to_metro,
        a.distance_to_bus_stop,
        a.distance_to_shopping,
        a.created_at AS apartment_created_at,
        a.updated_at AS apartment_updated_at,

        -- address fields ...
        ad.id AS address_id,
        ad.street_address,
        ad.city,
        ad.state_or_region,
        ad.county_or_district,
        ad.postal_code,
        ad.country,
        ad.latitude,
        ad.longitude,
        ad.created_at AS address_created_at,
        ad.updated_at AS address_updated_at,

        COALESCE(
            (SELECT jsonb_agg(jsonb_build_object(
                'id', lt.id,
                'listing_id', lt.listing_id,
                'tag', lt.tag,
                'created_at', lt.created_at,
                'updated_at', lt.updated_at
            )) FROM listing_tags lt WHERE lt.listing_id = l.id),
            '[]'::jsonb
        ) AS "tags",

        COALESCE(
            (SELECT jsonb_agg(jsonb_build_object(
                'id', aa.id,
                'apartment_id', aa.apartment_id,
                'amenity', aa.amenity,
                'created_at', aa.created_at,
                'updated_at', aa.updated_at
            )) FROM apartment_amenities aa WHERE aa.apartment_id = a.id),
            '[]'::jsonb
        ) AS "amenities",

        COALESCE(
            (SELECT jsonb_agg(jsonb_build_object(
                'id', ap.id,
                'apartment_id', ap.apartment_id,
                'url', ap.url,
                'is_primary', ap.is_primary,
                'created_at', ap.created_at,
                'updated_at', ap.updated_at
            )) FROM apartment_pictures ap WHERE ap.apartment_id = a.id),
            '[]'::jsonb
        ) AS "pictures"
    "#;

    let mut listing_qb = QueryBuilder::new(select_base);
    let mut count_qb = QueryBuilder::new(
        r#"
        SELECT COUNT(*)
        FROM listings l
        JOIN users u ON u.id = l.owner_id
        JOIN apartments a ON a.id = l.apartment_id
        LEFT JOIN addresses ad ON ad.apartment_id = a.id
        WHERE 1=1
        "#,
    );

    listing_qb.push(
        r#"
        FROM listings l
        JOIN users u ON u.id = l.owner_id
        JOIN apartments a ON a.id = l.apartment_id
        LEFT JOIN addresses ad ON ad.apartment_id = a.id
        WHERE 1=1
        "#,
    );

    // Apply vector search filter if available
    if let Some(ids) = &vector_matched_ids {
        if !ids.is_empty() {
            listing_qb.push(" AND l.id = ANY(");
            listing_qb.push_bind(ids);
            listing_qb.push(")");

            count_qb.push(" AND l.id = ANY(");
            count_qb.push_bind(ids);
            count_qb.push(")");
        } else {
            // No matches from vector search, return empty
            return Ok((vec![], 0));
        }
    }

    if !search_params.country.trim().is_empty() {
        listing_qb
            .push(" AND ad.country = ")
            .push_bind(search_params.country.clone());
        count_qb
            .push(" AND ad.country = ")
            .push_bind(search_params.country.clone());
    }

    if let Some(max_price) = search_params.max_price {
        if max_price > 0 {
            listing_qb
                .push(" AND l.price <= ")
                .push_bind(max_price.to_string())
                .push("::numeric");
            count_qb
                .push(" AND l.price <= ")
                .push_bind(max_price.to_string())
                .push("::numeric");
        }
    }

    if let Some(min_rooms) = search_params.min_rooms {
        if min_rooms > 0 {
            listing_qb.push(" AND a.rooms >= ").push_bind(min_rooms);
            count_qb.push(" AND a.rooms >= ").push_bind(min_rooms);
        }
    }

    if let Some(min_beds) = search_params.min_beds {
        if min_beds > 0 {
            listing_qb.push(" AND a.beds >= ").push_bind(min_beds);
            count_qb.push(" AND a.beds >= ").push_bind(min_beds);
        }
    }

    if let Some(min_baths) = search_params.min_baths {
        if min_baths > 0 {
            listing_qb.push(" AND a.baths >= ").push_bind(min_baths);
            count_qb.push(" AND a.baths >= ").push_bind(min_baths);
        }
    }

    if let Some(min_area) = search_params.min_area {
        if min_area > 0 {
            listing_qb.push(" AND a.area >= ").push_bind(min_area);
            count_qb.push(" AND a.area >= ").push_bind(min_area);
        }
    }

    if let Some(apartment_floor) = search_params.apartment_floor {
        listing_qb
            .push(" AND a.apartment_floor = ")
            .push_bind(apartment_floor);
        count_qb
            .push(" AND a.apartment_floor = ")
            .push_bind(apartment_floor);
    }

    if let Some(min_building_floors) = search_params.min_building_floors {
        if min_building_floors > 0 {
            listing_qb
                .push(" AND a.total_building_floors >= ")
                .push_bind(min_building_floors);
            count_qb
                .push(" AND a.total_building_floors >= ")
                .push_bind(min_building_floors);
        }
    }

    if let Some(condition) = &search_params.condition {
        let condition_str = match condition {
            ApartmentCondition::Old => "old",
            ApartmentCondition::Repaired => "repaired",
            ApartmentCondition::New => "new",
        };
        listing_qb
            .push(" AND a.condition = ")
            .push_bind(condition_str)
            .push("::apartment_condition");
        count_qb
            .push(" AND a.condition = ")
            .push_bind(condition_str)
            .push("::apartment_condition");
    }

    if let Some(sale_type) = &search_params.sale_type {
        let sale_type_str = match sale_type {
            SaleType::Buy => "buy",
            SaleType::Rent => "rent",
        };
        listing_qb
            .push(" AND a.sale_type = ")
            .push_bind(sale_type_str)
            .push("::sale_type");
        count_qb
            .push(" AND a.sale_type = ")
            .push_bind(sale_type_str)
            .push("::sale_type");
    }

    // Boolean filters
    if let Some(furnished) = search_params.furnished {
        listing_qb.push(" AND a.furnished = ").push_bind(furnished);
        count_qb.push(" AND a.furnished = ").push_bind(furnished);
    }

    if let Some(pets_allowed) = search_params.pets_allowed {
        listing_qb
            .push(" AND a.pets_allowed = ")
            .push_bind(pets_allowed);
        count_qb
            .push(" AND a.pets_allowed = ")
            .push_bind(pets_allowed);
    }

    if let Some(has_elevator) = search_params.has_elevator {
        listing_qb
            .push(" AND a.has_elevator = ")
            .push_bind(has_elevator);
        count_qb
            .push(" AND a.has_elevator = ")
            .push_bind(has_elevator);
    }

    if let Some(has_garden) = search_params.has_garden {
        listing_qb
            .push(" AND a.has_garden = ")
            .push_bind(has_garden);
        count_qb.push(" AND a.has_garden = ").push_bind(has_garden);
    }

    if let Some(has_parking) = search_params.has_parking {
        listing_qb
            .push(" AND a.has_parking = ")
            .push_bind(has_parking);
        count_qb
            .push(" AND a.has_parking = ")
            .push_bind(has_parking);
    }

    if let Some(has_balcony) = search_params.has_balcony {
        listing_qb
            .push(" AND a.has_balcony = ")
            .push_bind(has_balcony);
        count_qb
            .push(" AND a.has_balcony = ")
            .push_bind(has_balcony);
    }

    if let Some(has_ac) = search_params.has_ac {
        listing_qb.push(" AND a.has_ac = ").push_bind(has_ac);
        count_qb.push(" AND a.has_ac = ").push_bind(has_ac);
    }

    if let Some(has_heating) = search_params.has_heating {
        listing_qb
            .push(" AND a.has_heating = ")
            .push_bind(has_heating);
        count_qb
            .push(" AND a.has_heating = ")
            .push_bind(has_heating);
    }

    // Distance filters
    if let Some(max_distance) = search_params.max_distance_to_kindergarten {
        if max_distance > 0 {
            listing_qb
                .push(" AND a.distance_to_kindergarten <= ")
                .push_bind(max_distance);
            count_qb
                .push(" AND a.distance_to_kindergarten <= ")
                .push_bind(max_distance);
        }
    }

    if let Some(max_distance) = search_params.max_distance_to_school {
        if max_distance > 0 {
            listing_qb
                .push(" AND a.distance_to_school <= ")
                .push_bind(max_distance);
            count_qb
                .push(" AND a.distance_to_school <= ")
                .push_bind(max_distance);
        }
    }

    if let Some(max_distance) = search_params.max_distance_to_hospital {
        if max_distance > 0 {
            listing_qb
                .push(" AND a.distance_to_hospital <= ")
                .push_bind(max_distance);
            count_qb
                .push(" AND a.distance_to_hospital <= ")
                .push_bind(max_distance);
        }
    }

    if let Some(max_distance) = search_params.max_distance_to_metro {
        if max_distance > 0 {
            listing_qb
                .push(" AND a.distance_to_metro <= ")
                .push_bind(max_distance);
            count_qb
                .push(" AND a.distance_to_metro <= ")
                .push_bind(max_distance);
        }
    }

    if let Some(max_distance) = search_params.max_distance_to_bus_stop {
        if max_distance > 0 {
            listing_qb
                .push(" AND a.distance_to_bus_stop <= ")
                .push_bind(max_distance);
            count_qb
                .push(" AND a.distance_to_bus_stop <= ")
                .push_bind(max_distance);
        }
    }

    if let Some(max_distance) = search_params.max_distance_to_shopping {
        if max_distance > 0 {
            listing_qb
                .push(" AND a.distance_to_shopping <= ")
                .push_bind(max_distance);
            count_qb
                .push(" AND a.distance_to_shopping <= ")
                .push_bind(max_distance);
        }
    }

    // GROUP BY clause required for the aggregates
    listing_qb.push(" GROUP BY l.id, u.id, a.id, ad.id ");

    // Sorting - When vector search is active, preserve vector search order
    if vector_matched_ids.is_some() {
        // Create a custom ordering based on the vector search results
        // This maintains the relevance ranking from the hybrid search
        if let Some(ids) = &vector_matched_ids {
            if !ids.is_empty() {
                listing_qb.push(" ORDER BY array_position(ARRAY[");
                for (i, id) in ids.iter().enumerate() {
                    if i > 0 {
                        listing_qb.push(",");
                    }
                    listing_qb.push_bind(id);
                    listing_qb.push("::uuid");
                }
                listing_qb.push("], l.id)");
            }
        }
    } else if let Some(sort) = &search_params.sort {
        // Regular sorting when no vector search
        match sort {
            Sort::Newest => listing_qb.push(" ORDER BY l.created_at DESC "),
            Sort::Cheap => listing_qb.push(" ORDER BY l.price ASC "),
            Sort::Expensive => listing_qb.push(" ORDER BY l.price DESC "),
        };
    } else {
        // Default sorting
        listing_qb.push(" ORDER BY l.created_at DESC ");
    }

    listing_qb.push(" OFFSET ").push_bind(pagination.offset);
    listing_qb.push(" LIMIT ").push_bind(pagination.limit);

    let total = count_qb.build_query_scalar::<i64>().fetch_one(pool).await?;

    let rows: Vec<ListingJoined> = listing_qb
        .build_query_as::<ListingJoined>()
        .fetch_all(pool)
        .await?;

    let listings = rows
        .into_iter()
        .map(|row| ListingOut {
            id: row.listing_id,
            price: row.price,
            created_at: row.listing_created_at,
            updated_at: row.listing_updated_at,
            owner: UserOut {
                id: row.owner_id,
                full_name: row.owner_full_name,
                email: row.owner_email,
                phone_number: row.owner_phone,
                picture: row.owner_picture,
                role: row.owner_role,
                status: row.owner_status,
                email_verified: row.owner_email_verified,
                oauth_user_id: row.owner_oauth_user_id,
                created_at: row.owner_created_at,
                updated_at: row.owner_updated_at,
            },
            apartment: ApartmentOut {
                id: row.apartment_id,
                title: row.apartment_title,
                description: row.apartment_description,
                rooms: row.apartment_rooms,
                beds: row.apartment_beds,
                baths: row.apartment_baths,
                address: Some(AddressOut {
                    id: row.address_id,
                    apartment_id: row.apartment_id,
                    street_address: row.street_address,
                    city: row.city,
                    state_or_region: row.state_or_region,
                    county_or_district: row.county_or_district,
                    postal_code: row.postal_code,
                    country: row.country,
                    latitude: row.latitude,
                    longitude: row.longitude,
                    created_at: row.address_created_at,
                    updated_at: row.address_updated_at,
                }),
                pictures: row.pictures.map(|p| p.0).unwrap_or_default(),
                amenities: row.amenities.map(|a| a.0).unwrap_or_default(),
                area: row.apartment_area,
                apartment_floor: row.apartment_floor,
                total_building_floors: row.apartment_total_building_floors,
                condition: row.apartment_condition,
                sale_type: row.apartment_sale_type,
                requirements: row.apartment_requirements,
                furnished: row.apartment_furnished,
                pets_allowed: row.apartment_pets_allowed,
                has_elevator: row.apartment_has_elevator,
                has_garden: row.apartment_has_garden,
                has_parking: row.apartment_has_parking,
                has_balcony: row.apartment_has_balcony,
                has_ac: row.apartment_has_ac,
                has_heating: row.apartment_has_heating,
                distance_to_kindergarten: row.distance_to_kindergarten,
                distance_to_school: row.distance_to_school,
                distance_to_hospital: row.distance_to_hospital,
                distance_to_metro: row.distance_to_metro,
                distance_to_bus_stop: row.distance_to_bus_stop,
                distance_to_shopping: row.distance_to_shopping,
                created_at: row.apartment_created_at,
                updated_at: row.apartment_updated_at,
            },
            tags: row.tags.map(|t| t.0).unwrap_or_default(),
        })
        .collect();

    // https://api.frankfurter.dev/v1/latest?base=USD&symbols=EUR
    // {
    //   "base": "USD",
    //   "date": "2025-10-13",
    //   "rates": {
    //     "EUR": 0.9304
    //   }
    // }

    Ok((listings, total))
}

async fn perform_hybrid_search(
    query: &str,
    country: String,
    qdrant: &Qdrant,
    ai: &AI,
) -> Result<Option<Vec<Uuid>>, AppError> {
    debug!("Starting hybrid search for query: '{}'", query);

    // Generate text embedding
    let text_embedding = ai.embed_text(query)?;
    debug!(
        "Generated text embedding with {} dimensions",
        text_embedding.len()
    );

    let mut conds = vec![];
    if !country.trim().is_empty() {
        conds.push(Condition::matches("country", country));
    }
    let filter = Filter::must(conds);

    // Use Qdrant's Query API with fusion for hybrid search
    // This combines text and image vector searches with proper scoring
    let query_request = QueryPointsBuilder::new("listings")
        .add_prefetch(
            PrefetchQueryBuilder::default()
                .query(text_embedding.clone())
                // .query(Query::new_nearest(text_embedding.clone()))
                .using("text")
                .limit(100u64), // Get top 100 from text search
        )
        .add_prefetch(
            PrefetchQueryBuilder::default()
                .query(text_embedding)
                // .query(Query::new_nearest(text_embedding))
                .using("image")
                .limit(50u64), // Get top 50 from image search
        )
        .limit(50u64) // Final limit after fusion
        .filter(filter)
        .with_payload(true)
        .query(Query::new_fusion(Fusion::Rrf)); // Reciprocal Rank Fusion

    // Execute the query
    let search_result = qdrant.query(query_request.build()).await.map_err(|e| {
        debug!("Qdrant query failed: {:?}", e);
        AppError::VectorSearchError(e.to_string())
    })?;

    debug!(
        "Hybrid search returned {} results",
        search_result.result.len()
    );

    if search_result.result.is_empty() {
        debug!("No results found in hybrid search");
        return Ok(None);
    }

    // Extract listing IDs preserving the relevance order from fusion
    let listing_ids: Vec<Uuid> = search_result
        .result
        .into_iter()
        .filter_map(|point| {
            point.id.and_then(|id| {
                let id_str = match id.point_id_options {
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) => uuid,
                    Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => {
                        num.to_string()
                    }
                    None => return None,
                };

                Uuid::parse_str(&id_str).ok()
            })
        })
        .collect();

    debug!(
        "Hybrid search returning {} unique listings in relevance order",
        listing_ids.len()
    );

    Ok(Some(listing_ids))

    // let searches = vec![
    //     QueryPointsBuilder::new("listings")
    //         .query(text_embedding)
    //         .limit(20)
    //         .filter(filter.clone())
    //         .using("text")
    //         .build(),
    //     QueryPointsBuilder::new("listings")
    //         .query(VectorInput::new_multi(vec![text_embedding]))
    //         .limit(20)
    //         .filter(filter)
    //         .using("image")
    //         .build(),
    // ];

    // let query_batch_response = qdrant
    //     .query_batch(QueryBatchPointsBuilder::new("listings", searches))
    //     .await?;

    // // Search using text vector
    // let text_search = qdrant
    //     .search_points(
    //         SearchPointsBuilder::new("listings", text_embedding.clone(), 50)
    //             .filter(Filter::must(conds.clone()))
    //             .vector_name("text")
    //             .with_payload(true)
    //             .score_threshold(0.3),
    //     )
    //     .await?;
}

pub async fn get_one_listing(pool: &PgPool, listing_id: &Uuid) -> Result<ListingOut, AppError> {
    let row = sqlx::query_as!(
        ListingJoined,
        r#"
        SELECT
            l.id AS listing_id,
            l.price,
            l.created_at AS listing_created_at,
            l.updated_at AS listing_updated_at,

            -- owner fields ...
            u.id AS owner_id,
            u.full_name AS owner_full_name,
            u.email AS owner_email,
            u.phone_number AS owner_phone,
            u.picture AS owner_picture,
            u.role AS "owner_role: UserRole",
            u.status AS "owner_status: UserStatus",
            u.email_verified AS owner_email_verified,
            u.oauth_user_id AS owner_oauth_user_id,
            u.created_at AS owner_created_at,
            u.updated_at AS owner_updated_at,

            -- apartment fields ...
            a.id AS apartment_id,
            a.title AS apartment_title,
            a.description AS apartment_description,
            a.rooms AS apartment_rooms,
            a.beds AS apartment_beds,
            a.baths AS apartment_baths,
            a.area AS apartment_area,
            a.apartment_floor AS apartment_floor,
            a.total_building_floors AS apartment_total_building_floors,
            a.condition AS "apartment_condition: ApartmentCondition",
            a.sale_type AS "apartment_sale_type: SaleType",
            a.requirements AS apartment_requirements,
            a.furnished AS apartment_furnished,
            a.pets_allowed AS apartment_pets_allowed,
            a.has_elevator AS apartment_has_elevator,
            a.has_garden AS apartment_has_garden,
            a.has_parking AS apartment_has_parking,
            a.has_balcony AS apartment_has_balcony,
            a.has_ac AS apartment_has_ac,
            a.has_heating AS apartment_has_heating,
            a.distance_to_kindergarten,
            a.distance_to_school,
            a.distance_to_hospital,
            a.distance_to_metro,
            a.distance_to_bus_stop,
            a.distance_to_shopping,
            a.created_at AS apartment_created_at,
            a.updated_at AS apartment_updated_at,

            -- address fields ...
            ad.id AS address_id,
            ad.street_address,
            ad.city,
            ad.state_or_region,
            ad.county_or_district,
            ad.postal_code,
            ad.country,
            ad.latitude,
            ad.longitude,
            ad.created_at AS address_created_at,
            ad.updated_at AS address_updated_at,

            COALESCE(
                (SELECT jsonb_agg(jsonb_build_object(
                    'id', lt.id,
                    'listing_id', lt.listing_id,
                    'tag', lt.tag,
                    'created_at', lt.created_at,
                    'updated_at', lt.updated_at
                )) FROM listing_tags lt WHERE lt.listing_id = l.id),
                '[]'::jsonb
            ) AS "tags: Json<Vec<TagOut>>",

            COALESCE(
                (SELECT jsonb_agg(jsonb_build_object(
                    'id', aa.id,
                    'apartment_id', aa.apartment_id,
                    'amenity', aa.amenity,
                    'created_at', aa.created_at,
                    'updated_at', aa.updated_at
                )) FROM apartment_amenities aa WHERE aa.apartment_id = a.id),
                '[]'::jsonb
            ) AS "amenities: Json<Vec<AmenityOut>>",

            COALESCE(
                (SELECT jsonb_agg(jsonb_build_object(
                    'id', ap.id,
                    'apartment_id', ap.apartment_id,
                    'url', ap.url,
                    'is_primary', ap.is_primary,
                    'created_at', ap.created_at,
                    'updated_at', ap.updated_at
                )) FROM apartment_pictures ap WHERE ap.apartment_id = a.id),
                '[]'::jsonb
            ) AS "pictures: Json<Vec<PictureOut>>"

        FROM listings l
        JOIN users u ON u.id = l.owner_id
        JOIN apartments a ON a.id = l.apartment_id
        LEFT JOIN addresses ad ON ad.apartment_id = a.id
        WHERE l.id = $1
        GROUP BY l.id, u.id, a.id, ad.id
        ORDER BY l.created_at DESC
        "#,
        &listing_id
    )
    .fetch_one(pool)
    .await
    .map_err(|_e| AppError::DatabaseFetchError {
        resource: "Model".to_string(),
        id: listing_id.to_string(),
    })?;

    let listing = ListingOut {
        id: row.listing_id,
        price: row.price,
        created_at: row.listing_created_at,
        updated_at: row.listing_updated_at,
        owner: UserOut {
            id: row.owner_id,
            full_name: row.owner_full_name,
            email: row.owner_email,
            phone_number: row.owner_phone,
            picture: row.owner_picture,
            role: row.owner_role,
            status: row.owner_status,
            email_verified: row.owner_email_verified,
            oauth_user_id: row.owner_oauth_user_id,
            created_at: row.owner_created_at,
            updated_at: row.owner_updated_at,
        },
        apartment: ApartmentOut {
            id: row.apartment_id,
            title: row.apartment_title,
            description: row.apartment_description,
            rooms: row.apartment_rooms,
            beds: row.apartment_beds,
            baths: row.apartment_baths,
            address: Some(AddressOut {
                id: row.address_id,
                apartment_id: row.apartment_id,
                street_address: row.street_address,
                city: row.city,
                state_or_region: row.state_or_region,
                county_or_district: row.county_or_district,
                postal_code: row.postal_code,
                country: row.country,
                latitude: row.latitude,
                longitude: row.longitude,
                created_at: row.address_created_at,
                updated_at: row.address_updated_at,
            }),
            pictures: row.pictures.map(|p| p.0).unwrap_or_default(),
            amenities: row.amenities.map(|a| a.0).unwrap_or_default(),
            area: row.apartment_area,
            apartment_floor: row.apartment_floor,
            total_building_floors: row.apartment_total_building_floors,
            condition: row.apartment_condition,
            sale_type: row.apartment_sale_type,
            requirements: row.apartment_requirements,
            furnished: row.apartment_furnished,
            pets_allowed: row.apartment_pets_allowed,
            has_elevator: row.apartment_has_elevator,
            has_garden: row.apartment_has_garden,
            has_parking: row.apartment_has_parking,
            has_balcony: row.apartment_has_balcony,
            has_ac: row.apartment_has_ac,
            has_heating: row.apartment_has_heating,
            distance_to_kindergarten: row.distance_to_kindergarten,
            distance_to_school: row.distance_to_school,
            distance_to_hospital: row.distance_to_hospital,
            distance_to_metro: row.distance_to_metro,
            distance_to_bus_stop: row.distance_to_bus_stop,
            distance_to_shopping: row.distance_to_shopping,
            created_at: row.apartment_created_at,
            updated_at: row.apartment_updated_at,
        },
        tags: row.tags.map(|t| t.0).unwrap_or_default(),
    };

    Ok(listing)
}

pub async fn create_listing(
    owner_id: Uuid,
    pool: &PgPool,
    gcs: GoogleCloudStorage,
    qdrant: Qdrant,
    ai: AI,
    mut multipart: Multipart,
) -> Result<(), AppError> {
    let mut listing_json: Option<String> = None;
    let mut picture_files: Vec<bytes::Bytes> = Vec::new();

    // Parse multipart safely
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::InvalidFormData("Failed to read multipart stream".into()))?
    {
        debug!(
            "name: {:?}, file_name: {:?}",
            field.name(),
            field.file_name()
        );
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "listing_data" => {
                let text = field.text().await.map_err(|_| {
                    AppError::InvalidFormData("Failed to read listing_data field".into())
                })?;
                listing_json = Some(text);
            }
            "pictures" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| AppError::InvalidFormData("Failed to read picture file".into()))?;
                picture_files.push(bytes);
            }
            _ => {
                warn!("Unknown multipart field: {}", name);
            }
        }
    }

    debug!("listing_json: {:#?}", listing_json);

    let listing_in: ListingIn = serde_json::from_str(&listing_json.unwrap())?;

    debug!("listing_in: {:#?}", listing_in);

    // Start transaction
    let mut tx = pool.begin().await?;

    // Create apartment
    let apartment_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO apartments (
            id,
            title,
            description,
            rooms,
            beds,
            baths,
            area,
            apartment_floor,
            total_building_floors,
            condition,
            sale_type,
            requirements,
            furnished,
            pets_allowed,
            has_elevator,
            has_garden,
            has_parking,
            has_balcony,
            has_ac,
            has_heating,
            distance_to_kindergarten,
            distance_to_school,
            distance_to_hospital,
            distance_to_metro,
            distance_to_bus_stop,
            distance_to_shopping)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)
        "#,
        apartment_id,
        listing_in.apartment.title,
        listing_in.apartment.description,
        listing_in.apartment.rooms,
        listing_in.apartment.beds,
        listing_in.apartment.baths,
        listing_in.apartment.area,
        listing_in.apartment.apartment_floor,
        listing_in.apartment.total_building_floors,
        listing_in.apartment.condition as ApartmentCondition, // this correct
        // SQLx doesn’t automatically know how to encode your enum
        listing_in.apartment.sale_type as _, // this also correct, let Rust infer SQLx type for enums
        listing_in.apartment.requirements,
        listing_in.apartment.furnished,
        listing_in.apartment.pets_allowed,
        listing_in.apartment.has_elevator,
        listing_in.apartment.has_garden,
        listing_in.apartment.has_parking,
        listing_in.apartment.has_balcony,
        listing_in.apartment.has_ac,
        listing_in.apartment.has_heating,
        listing_in.apartment.distance_to_kindergarten,
        listing_in.apartment.distance_to_school,
        listing_in.apartment.distance_to_hospital,
        listing_in.apartment.distance_to_metro,
        listing_in.apartment.distance_to_bus_stop,
        listing_in.apartment.distance_to_shopping
    )
    .execute(&mut *tx)
    .await?;

    // Create address
    sqlx::query!(
        r#"
        INSERT INTO addresses (apartment_id, street_address, city, state_or_region, 
            county_or_district, postal_code, country, latitude, longitude)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        "#,
        apartment_id,
        listing_in.apartment.address.street_address,
        listing_in.apartment.address.city,
        listing_in.apartment.address.state_or_region,
        listing_in.apartment.address.county_or_district,
        listing_in.apartment.address.postal_code,
        listing_in.apartment.address.country,
        listing_in.apartment.address.latitude,
        listing_in.apartment.address.longitude
    )
    .execute(&mut *tx)
    .await?;

    for amenity in listing_in.apartment.amenities {
        sqlx::query!(
            r#"
            INSERT INTO apartment_amenities (apartment_id, amenity)
            VALUES ($1,$2)
            "#,
            apartment_id,
            amenity
        )
        .execute(&mut *tx)
        .await?;
    }

    // Upload pictures and insert records
    for (idx, data) in picture_files.iter().enumerate() {
        let pic_id = Uuid::new_v4();
        let ext = infer::get(data)
            .ok_or_else(|| AppError::InvalidImageFormatError("Invalid image format".to_string()))?
            .extension();

        let location = ObjectStorePath::from(format!("{}/{}.{}", apartment_id, pic_id, ext));
        gcs.put(&location, data.clone().into()).await?;

        let is_primary = idx == 0; // First picture is primary, at least for now, we can do something later
        sqlx::query!(
            "INSERT INTO apartment_pictures (id, apartment_id, url, is_primary) VALUES ($1, $2, $3, $4)",
            pic_id,
            apartment_id,
            location.to_string(),
            is_primary
        )
        .execute(&mut *tx)
        .await?;
    }

    // Create listing
    let listing_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO listings (id, apartment_id, owner_id, price) VALUES ($1, $2, $3, $4)",
        listing_id,
        apartment_id,
        owner_id,
        listing_in.price,
    )
    .execute(&mut *tx)
    .await?;

    // Listing tags
    for tag in listing_in.tags {
        if tag.is_empty() {
            continue;
        };

        sqlx::query!(
            "INSERT INTO listing_tags (listing_id, tag) VALUES ($1, $2)",
            listing_id,
            tag
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let point_id = PointId::from(listing_id.to_string());

    let text_content = format!(
        "{} {}",
        listing_in.apartment.title,
        listing_in.apartment.description.as_deref().unwrap_or("")
    );
    let text_vector = ai.embed_text(&text_content.trim())?;

    let mut image_vectors: Vec<Vec<f32>> = Vec::new();
    for bytes in &picture_files {
        match ai.embed_image_bytes(bytes) {
            Ok(vec) => image_vectors.push(vec),
            Err(e) => {
                debug!("Failed to embed image, skipping: {}", e);
            }
        }
    }

    let named_vectors = NamedVectors::default()
        .add_vector("text", Vector::new_dense(text_vector))
        .add_vector("image", Vector::new_multi(image_vectors));

    let payload = Payload::try_from(json!({
        "listing_id": listing_id.to_string(),
        "country": listing_in.apartment.address.country
    }))?;

    let points = vec![PointStruct::new(point_id, named_vectors, payload)];

    qdrant
        .upsert_points(UpsertPointsBuilder::new("listings", points).wait(true))
        .await?;

    Ok(())
}
