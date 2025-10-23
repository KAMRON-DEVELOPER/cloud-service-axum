use crate::utilities::{config::Config, errors::AppError};
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, Datatype, Distance, MultiVectorComparator,
        MultiVectorConfigBuilder, SparseVectorParamsBuilder, SparseVectorsConfigBuilder,
        VectorParamsBuilder, VectorsConfigBuilder,
    },
};
use tracing::info;

pub async fn build_qdrant(config: &Config) -> Result<Qdrant, AppError> {
    let client = Qdrant::from_url(
        &config
            .qdrant_url
            .as_ref()
            .ok_or_else(|| AppError::MissingQdrantUrlError)?,
    )
    .api_key(
        config
            .qdrant_api_key
            .clone()
            .ok_or_else(|| AppError::MissingQdrantApiKeyError)?,
    )
    .timeout(std::time::Duration::from_secs(10))
    .build()?;

    let health_check_reply = client.health_check().await?;

    info!("Qdrant health check reply: {:#?}", health_check_reply);

    // Initialize collections
    initialize_collections(&client).await?;

    Ok(client)
}

async fn initialize_collections(client: &Qdrant) -> Result<(), AppError> {
    // Text embeddings collection (512 dimensions for clip-ViT-B-32(-text))
    // let text_collection_name = "listings_text";
    // if !client.collection_exists(text_collection_name).await? {
    //     info!("Creating Qdrant text collection: {}", text_collection_name);
    //     client
    //         .create_collection(
    //             CreateCollectionBuilder::new(text_collection_name)
    //                 .vectors_config(VectorParamsBuilder::new(512, Distance::Cosine).build()),
    //         )
    //         .await?;
    // }

    // Image embeddings collection (512 dimensions for clip-ViT-B-32(-vision))
    // let image_collection_name = "listings_images";
    // if !client.collection_exists(image_collection_name).await? {
    //     info!(
    //         "Creating Qdrant image collection: {}",
    //         image_collection_name
    //     );
    //     client
    //         .create_collection(
    //             CreateCollectionBuilder::new(image_collection_name)
    //                 .vectors_config(VectorParamsBuilder::new(512, Distance::Cosine).build()),
    //         )
    //         .await?;
    // }

    let collection_name = "listings";
    if !client.collection_exists(collection_name).await? {
        info!("Creating Qdrant collection: {}", collection_name);

        let mut vector_config = VectorsConfigBuilder::default();
        vector_config.add_named_vector_params(
            "text",
            VectorParamsBuilder::new(512, Distance::Cosine).datatype(Datatype::Float32),
        );
        vector_config.add_named_vector_params(
            "image",
            VectorParamsBuilder::new(512, Distance::Cosine)
                .multivector_config(MultiVectorConfigBuilder::new(MultiVectorComparator::MaxSim))
                .datatype(Datatype::Float32),
        );

        let mut sparse_vectors_config = SparseVectorsConfigBuilder::default();
        sparse_vectors_config
            .add_named_vector_params("text-sparse", SparseVectorParamsBuilder::default());

        client
            .create_collection(
                CreateCollectionBuilder::new("listings")
                    .vectors_config(vector_config)
                    .sparse_vectors_config(sparse_vectors_config),
            )
            .await?;
    }

    Ok(())
}
