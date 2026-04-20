//! Tag repository tests
//!
//! Integration tests for tag CRUD operations, statistics, and suggestions

use iou_core::tag::{
    Tag, TagType, TagRepository, TagError, TagStats,
    TagSuggestion, SuggestionReason,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Test helper to create a test database pool
async fn create_test_pool() -> PgPool {
    // This would normally use a test database URL
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://iou_user:iou_password@localhost:5432/iou_modern_test".to_string());

    sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Test helper to create a test tag
async fn create_test_tag(
    repo: &TagRepository,
    name: &str,
    tag_type: TagType,
) -> Result<Tag, TagError> {
    repo.create_tag(
        name.to_string(),
        tag_type,
        None,
        Some(format!("Test tag: {}", name)),
        Some("#FF0000".to_string()),
        None,
        false,
    ).await
}

/// Test helper to create a test information object
async fn create_test_object(pool: &PgPool) -> Uuid {
    let object_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO information_objects (id, title, status, created_by)
        VALUES ($1, $2, 'draft', $3)
        "#
    )
    .bind(&object_id)
    .bind("Test Object")
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .expect("Failed to create test object");

    object_id
}

#[tokio::test]
async fn test_tag_repository_create_tag() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag = create_test_tag(&repo, "test-tag", TagType::User)
        .await
        .expect("Failed to create tag");

    assert_eq!(tag.name, "test-tag");
    assert_eq!(tag.tag_type, TagType::User);
    assert!(!tag.is_system_tag);
    assert_eq!(tag.usage_count, 0);
}

#[tokio::test]
async fn test_tag_repository_get_tag() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let created = create_test_tag(&repo, "get-test", TagType::User)
        .await
        .expect("Failed to create tag");

    let retrieved = repo.get_tag(created.id)
        .await
        .expect("Failed to get tag");

    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.name, "get-test");
}

#[tokio::test]
async fn test_tag_repository_get_tag_not_found() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool);

    let result = repo.get_tag(Uuid::new_v4()).await;

    assert!(matches!(result, Err(TagError::TagNotFound(_))));
}

#[tokio::test]
async fn test_tag_repository_get_tag_by_name() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let created_name = "name-test";
    create_test_tag(&repo, created_name, TagType::User)
        .await
        .expect("Failed to create tag");

    let retrieved = repo.get_tag_by_name(&created_name.to_lowercase())
        .await
        .expect("Failed to get tag by name");

    assert_eq!(retrieved.name, created_name);
}

#[tokio::test]
async fn test_tag_repository_list_tags() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    // Create test tags
    create_test_tag(&repo, "tag-1", TagType::User).await.expect("Failed to create tag 1");
    create_test_tag(&repo, "tag-2", TagType::Domain).await.expect("Failed to create tag 2");
    create_test_tag(&repo, "tag-3", TagType::User).await.expect("Failed to create tag 3");

    // List all user tags
    let user_tags = repo.list_tags(Some(TagType::User), None, 50, 0)
        .await
        .expect("Failed to list tags");

    assert!(user_tags.len() >= 2);

    // Check filtering
    let filtered_tags: Vec<_> = user_tags.iter()
        .filter(|t| t.name == "tag-1" || t.name == "tag-3")
        .collect();

    assert_eq!(filtered_tags.len(), 2);
}

#[tokio::test]
async fn test_tag_repository_search_tags() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    create_test_tag(&repo, "rust-programming", TagType::User).await.expect("Failed to create tag");
    create_test_tag(&repo, "python-script", TagType::User).await.expect("Failed to create tag");

    let results = repo.search_tags("rust", 10)
        .await
        .expect("Failed to search tags");

    assert!(results.iter().any(|t| t.name.contains("rust")));
}

#[tokio::test]
async fn test_tag_repository_update_tag() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag = create_test_tag(&repo, "update-test", TagType::User)
        .await
        .expect("Failed to create tag");

    let updated = repo.update_tag(
        tag.id,
        Some("updated-name".to_string()),
        Some(Some("Updated description".to_string())),
        Some(Some("#00FF00".to_string())),
    ).await
    .expect("Failed to update tag");

    assert_eq!(updated.name, "updated-name");
    assert_eq!(updated.description, Some("Updated description".to_string()));
    assert_eq!(updated.color, Some("#00FF00".to_string()));
}

#[tokio::test]
async fn test_tag_repository_delete_tag() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag = create_test_tag(&repo, "delete-test", TagType::User)
        .await
        .expect("Failed to create tag");

    repo.delete_tag(tag.id)
        .await
        .expect("Failed to delete tag");

    let result = repo.get_tag(tag.id).await;
    assert!(matches!(result, Err(TagError::TagNotFound(_))));
}

#[tokio::test]
async fn test_tag_repository_tag_object() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag = create_test_tag(&repo, "object-tag", TagType::User)
        .await
        .expect("Failed to create tag");

    let object_id = create_test_object(&pool).await;
    let user_id = Uuid::new_v4();

    let object_tag = repo.tag_object(object_id, tag.id, user_id)
        .await
        .expect("Failed to tag object");

    assert_eq!(object_tag.object_id, object_id);
    assert_eq!(object_tag.tag_id, tag.id);

    // Verify usage count increased
    let updated_tag = repo.get_tag(tag.id).await.expect("Failed to get tag");
    assert_eq!(updated_tag.usage_count, 1);
}

#[tokio::test]
async fn test_tag_repository_untag_object() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag = create_test_tag(&repo, "untag-test", TagType::User)
        .await
        .expect("Failed to create tag");

    let object_id = create_test_object(&pool).await;
    let user_id = Uuid::new_v4();

    // First tag the object
    repo.tag_object(object_id, tag.id, user_id)
        .await
        .expect("Failed to tag object");

    // Then untag
    repo.untag_object(object_id, tag.id)
        .await
        .expect("Failed to untag object");

    // Verify usage count decreased
    let updated_tag = repo.get_tag(tag.id).await.expect("Failed to get tag");
    assert_eq!(updated_tag.usage_count, 0);
}

#[tokio::test]
async fn test_tag_repository_get_object_tags() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag1 = create_test_tag(&repo, "obj-tag-1", TagType::User)
        .await
        .expect("Failed to create tag 1");
    let tag2 = create_test_tag(&repo, "obj-tag-2", TagType::User)
        .await
        .expect("Failed to create tag 2");

    let object_id = create_test_object(&pool).await;
    let user_id = Uuid::new_v4();

    repo.tag_object(object_id, tag1.id, user_id)
        .await
        .expect("Failed to tag object with tag1");
    repo.tag_object(object_id, tag2.id, user_id)
        .await
        .expect("Failed to tag object with tag2");

    let object_tags = repo.get_object_tags(object_id)
        .await
        .expect("Failed to get object tags");

    assert_eq!(object_tags.len(), 2);
}

#[tokio::test]
async fn test_tag_repository_tag_nonexistent_object() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool);

    let tag = create_test_tag(&repo, "error-test", TagType::User)
        .await
        .expect("Failed to create tag");

    let result = repo.tag_object(Uuid::new_v4(), tag.id, Uuid::new_v4()).await;

    assert!(matches!(result, Err(TagError::ObjectNotFound(_))));
}

#[tokio::test]
async fn test_tag_repository_get_tag_stats() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag = create_test_tag(&repo, "stats-test", TagType::User)
        .await
        .expect("Failed to create tag");

    let object_id = create_test_object(&pool).await;
    let user_id = Uuid::new_v4();

    repo.tag_object(object_id, tag.id, user_id)
        .await
        .expect("Failed to tag object");

    let stats = repo.get_tag_stats(tag.id)
        .await
        .expect("Failed to get tag stats");

    assert_eq!(stats.tag_id, tag.id);
    assert_eq!(stats.usage_count, 1);
    assert_eq!(stats.objects_count, 1);
}

#[tokio::test]
async fn test_tag_repository_get_popular_tags() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let tag1 = create_test_tag(&repo, "popular-1", TagType::User)
        .await
        .expect("Failed to create tag 1");
    let tag2 = create_test_tag(&repo, "popular-2", TagType::User)
        .await
        .expect("Failed to create tag 2");

    let object_id = create_test_object(&pool).await;
    let user_id = Uuid::new_v4();

    // Tag object multiple times with tag1
    repo.tag_object(object_id, tag1.id, user_id)
        .await
        .expect("Failed to tag object");

    let popular = repo.get_popular_tags(Some(10))
        .await
        .expect("Failed to get popular tags");

    assert!(!popular.is_empty());
}

#[tokio::test]
async fn test_tag_repository_merge_tags() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let source_tag = create_test_tag(&repo, "source-tag", TagType::User)
        .await
        .expect("Failed to create source tag");
    let target_tag = create_test_tag(&repo, "target-tag", TagType::User)
        .await
        .expect("Failed to create target tag");

    let object_id = create_test_object(&pool).await;
    let user_id = Uuid::new_v4();

    // Tag object with source
    repo.tag_object(object_id, source_tag.id, user_id)
        .await
        .expect("Failed to tag object");

    // Merge source into target
    repo.merge_tags(source_tag.id, target_tag.id)
        .await
        .expect("Failed to merge tags");

    // Source tag should be deleted
    let result = repo.get_tag(source_tag.id).await;
    assert!(matches!(result, Err(TagError::TagNotFound(_))));

    // Object should now have target tag
    let object_tags = repo.get_object_tags(object_id)
        .await
        .expect("Failed to get object tags");

    assert!(object_tags.iter().any(|t| t.id == target_tag.id));
}

#[tokio::test]
async fn test_tag_repository_get_tag_suggestions() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool.clone());

    let domain_id = Uuid::new_v4();

    let suggestions = repo.get_tag_suggestions(Some(domain_id), 5)
        .await
        .expect("Failed to get tag suggestions");

    // Should return suggestions even if empty
    assert!(suggestions.len() <= 5);
}

#[tokio::test]
async fn test_tag_repository_get_trending_tags() {
    let pool = create_test_pool().await;
    let repo = TagRepository::new(pool);

    let trending = repo.get_trending_tags(10, 7)
        .await
        .expect("Failed to get trending tags");

    // Should return tags even if empty
    assert!(trending.len() <= 10);
}
