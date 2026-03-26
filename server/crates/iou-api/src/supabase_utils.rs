//! Supabase Utility Functions
//!
//! Provides helper utilities for working with Supabase:
//! - Query builder for type-safe database queries
//! - Transaction helpers for atomic operations
//! - Pagination support
//! - Common database operations

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, postgres::PgRow};
use uuid::Uuid;

// ============================================
// Query Builder
// ============================================

/// Type-safe query builder for Supabase/PostgreSQL
#[derive(Clone)]
pub struct QueryBuilder {
    pool: PgPool,
    table: String,
    filters: Vec<FilterCondition>,
    order: Option<OrderBy>,
    limit: Option<i64>,
    offset: Option<i64>,
    select_columns: Option<Vec<String>>,
}

impl QueryBuilder {
    /// Create a new query builder for a table
    pub fn new(pool: PgPool, table: &str) -> Self {
        Self {
            pool,
            table: table.to_string(),
            filters: Vec::new(),
            order: None,
            limit: None,
            offset: None,
            select_columns: None,
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: &[&str]) -> Self {
        self.select_columns = Some(columns.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Add a filter condition
    pub fn filter(mut self, column: &str, operator: FilterOperator, value: Value) -> Self {
        self.filters.push(FilterCondition {
            column: column.to_string(),
            operator,
            value,
            logic: FilterLogic::And,
        });
        self
    }

    /// Add an OR filter condition
    pub fn or_filter(mut self, column: &str, operator: FilterOperator, value: Value) -> Self {
        self.filters.push(FilterCondition {
            column: column.to_string(),
            operator,
            value,
            logic: FilterLogic::Or,
        });
        self
    }

    /// Add equality filter (shorthand)
    pub fn eq(self, column: &str, value: impl Into<Value>) -> Self {
        self.filter(column, FilterOperator::Eq, value.into())
    }

    /// Add inequality filter
    pub fn neq(self, column: &str, value: impl Into<Value>) -> Self {
        self.filter(column, FilterOperator::Neq, value.into())
    }

    /// Add greater than filter
    pub fn gt(self, column: &str, value: impl Into<Value>) -> Self {
        self.filter(column, FilterOperator::Gt, value.into())
    }

    /// Add greater than or equal filter
    pub fn gte(self, column: &str, value: impl Into<Value>) -> Self {
        self.filter(column, FilterOperator::Gte, value.into())
    }

    /// Add less than filter
    pub fn lt(self, column: &str, value: impl Into<Value>) -> Self {
        self.filter(column, FilterOperator::Lt, value.into())
    }

    /// Add less than or equal filter
    pub fn lte(self, column: &str, value: impl Into<Value>) -> Self {
        self.filter(column, FilterOperator::Lte, value.into())
    }

    /// Add LIKE filter
    pub fn like(self, column: &str, pattern: &str) -> Self {
        self.filter(column, FilterOperator::Like, Value::String(pattern.to_string()))
    }

    /// Add IN filter
    pub fn r#in(self, column: &str, values: Vec<Value>) -> Self {
        self.filter(column, FilterOperator::In, Value::Array(values))
    }

    /// Add IS NULL filter
    pub fn is_null(self, column: &str) -> Self {
        self.filter(column, FilterOperator::IsNull, Value::Null)
    }

    /// Add ordering
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order = Some(OrderBy {
            column: column.to_string(),
            direction,
        });
        self
    }

    /// Add ascending order (shorthand)
    pub fn asc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Asc)
    }

    /// Add descending order (shorthand)
    pub fn desc(self, column: &str) -> Self {
        self.order_by(column, OrderDirection::Desc)
    }

    /// Set limit
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build and execute the query, returning raw rows
    pub async fn execute(&self) -> Result<Vec<PgRow>> {
        let (query, params) = self.build();
        let mut query_builder = sqlx::query(&query);

        for param in params {
            query_builder = match param {
                QueryParam::String(s) => query_builder.bind(s),
                QueryParam::Int(i) => query_builder.bind(i),
                QueryParam::Uuid(u) => query_builder.bind(u),
                QueryParam::Bool(b) => query_builder.bind(b),
                QueryParam::Null => query_builder.bind::<Option<String>>(None),
            };
        }

        let rows = query_builder.fetch_all(&self.pool).await?;
        Ok(rows)
    }

    /// Execute and return as JSON
    pub async fn fetch_json(&self) -> Result<Vec<Value>> {
        let rows = self.execute().await?;
        let results: Vec<Value> = rows
            .into_iter()
            .map(|row| row_to_json(row))
            .collect();

        Ok(results)
    }

    /// Execute and deserialize to a type
    pub async fn fetch<T: for<'de> Deserialize<'de>>(&self) -> Result<Vec<T>> {
        let json = self.fetch_json().await?;
        let items: Vec<T> = serde_json::from_value(Value::Array(json))?;
        Ok(items)
    }

    /// Execute and return first result
    pub async fn first<T: for<'de> Deserialize<'de>>(&self) -> Result<Option<T>> {
        let results = self.fetch().await?;
        Ok(results.into_iter().next())
    }

    /// Count matching records
    pub async fn count(&self) -> Result<i64> {
        let (query, params) = self.build_count();
        let mut query_builder = sqlx::query_scalar::<_, i64>(&query);

        for param in params {
            query_builder = match param {
                QueryParam::String(s) => query_builder.bind(s),
                QueryParam::Int(i) => query_builder.bind(i),
                QueryParam::Uuid(u) => query_builder.bind(u),
                QueryParam::Bool(b) => query_builder.bind(b),
                QueryParam::Null => query_builder.bind::<Option<String>>(None),
            };
        }

        let count = query_builder.fetch_one(&self.pool).await?;
        Ok(count)
    }

    /// Execute with pagination
    pub async fn paginate(&self, page: u32, per_page: u32) -> Result<PaginatedResult<Value>> {
        let offset = (page.saturating_sub(1) * per_page) as i64;
        let total = self.count().await?;

        // Create a new builder with limit and offset
        let paginated = QueryBuilder {
            pool: self.pool.clone(),
            table: self.table.clone(),
            filters: self.filters.clone(),
            order: self.order.clone(),
            limit: Some(per_page as i64),
            offset: Some(offset),
            select_columns: self.select_columns.clone(),
        };

        let items = paginated.fetch_json().await?;

        Ok(PaginatedResult {
            data: items,
            total,
            page,
            per_page,
            total_pages: ((total as f64) / (per_page as f64)).ceil() as u32,
        })
    }

    /// Build the SQL query
    fn build(&self) -> (String, Vec<QueryParam>) {
        let mut query = String::new();
        let mut params = Vec::new();

        // SELECT clause
        let columns = self.select_columns.as_ref()
            .map(|cols| cols.join(", "))
            .unwrap_or_else(|| "*".to_string());

        query.push_str(&format!("SELECT {} FROM {}", columns, self.table));

        // WHERE clause
        if !self.filters.is_empty() {
            query.push_str(" WHERE ");
            for (i, filter) in self.filters.iter().enumerate() {
                if i > 0 {
                    match filter.logic {
                        FilterLogic::And => query.push_str(" AND "),
                        FilterLogic::Or => query.push_str(" OR "),
                    }
                }

                match filter.operator {
                    FilterOperator::Eq => {
                        query.push_str(&format!("{} = ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Neq => {
                        query.push_str(&format!("{} != ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Gt => {
                        query.push_str(&format!("{} > ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Gte => {
                        query.push_str(&format!("{} >= ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Lt => {
                        query.push_str(&format!("{} < ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Lte => {
                        query.push_str(&format!("{} <= ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Like => {
                        query.push_str(&format!("{} LIKE ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::In => {
                        if let Value::Array(ref arr) = filter.value {
                            let placeholders: Vec<String> = arr
                                .iter()
                                .enumerate()
                                .map(|(j, _)| format!("${}", params.len() + j + 1))
                                .collect();
                            query.push_str(&format!("{} IN ({})", filter.column, placeholders.join(", ")));
                        }
                    }
                    FilterOperator::IsNull => {
                        query.push_str(&format!("{} IS NULL", filter.column));
                        continue; // Don't add parameter
                    }
                }

                params.push(value_to_param(&filter.value));
            }
        }

        // ORDER BY clause
        if let Some(ref order) = self.order {
            query.push_str(&format!(
                " ORDER BY {} {}",
                order.column,
                match order.direction {
                    OrderDirection::Asc => "ASC",
                    OrderDirection::Desc => "DESC",
                }
            ));
        }

        // LIMIT clause
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        (query, params)
    }

    /// Build count query
    fn build_count(&self) -> (String, Vec<QueryParam>) {
        let mut query = format!("SELECT COUNT(*) FROM {}", self.table);
        let mut params = Vec::new();

        if !self.filters.is_empty() {
            query.push_str(" WHERE ");
            for (i, filter) in self.filters.iter().enumerate() {
                if i > 0 {
                    match filter.logic {
                        FilterLogic::And => query.push_str(" AND "),
                        FilterLogic::Or => query.push_str(" OR "),
                    }
                }

                match filter.operator {
                    FilterOperator::Eq => {
                        query.push_str(&format!("{} = ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Neq => {
                        query.push_str(&format!("{} != ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Gt => {
                        query.push_str(&format!("{} > ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Gte => {
                        query.push_str(&format!("{} >= ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Lt => {
                        query.push_str(&format!("{} < ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Lte => {
                        query.push_str(&format!("{} <= ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::Like => {
                        query.push_str(&format!("{} LIKE ${}", filter.column, params.len() + 1));
                    }
                    FilterOperator::In => {
                        if let Value::Array(ref arr) = filter.value {
                            let placeholders: Vec<String> = arr
                                .iter()
                                .enumerate()
                                .map(|(j, _)| format!("${}", params.len() + j + 1))
                                .collect();
                            query.push_str(&format!("{} IN ({})", filter.column, placeholders.join(", ")));
                        }
                    }
                    FilterOperator::IsNull => {
                        query.push_str(&format!("{} IS NULL", filter.column));
                        continue;
                    }
                }

                params.push(value_to_param(&filter.value));
            }
        }

        (query, params)
    }
}

/// Filter condition for query builder
#[derive(Debug, Clone)]
struct FilterCondition {
    column: String,
    operator: FilterOperator,
    value: Value,
    logic: FilterLogic,
}

/// Filter operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    In,
    IsNull,
}

/// Filter logic (AND/OR)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterLogic {
    And,
    Or,
}

/// Order by clause
#[derive(Debug, Clone)]
struct OrderBy {
    column: String,
    direction: OrderDirection,
}

/// Order direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

/// Query parameter for prepared statements
#[derive(Debug, Clone)]
enum QueryParam {
    String(String),
    Int(i64),
    Uuid(Uuid),
    Bool(bool),
    Null,
}

/// Convert serde_json Value to QueryParam
fn value_to_param(value: &Value) -> QueryParam {
    match value {
        Value::String(s) => {
            // Try to parse as UUID first
            if let Ok(u) = Uuid::parse_str(s) {
                QueryParam::Uuid(u)
            } else {
                QueryParam::String(s.clone())
            }
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                QueryParam::Int(i)
            } else {
                QueryParam::String(n.to_string())
            }
        }
        Value::Bool(b) => QueryParam::Bool(*b),
        Value::Null => QueryParam::Null,
        Value::Array(arr) => {
            // For IN clause, add all values
            if let Some(first) = arr.first() {
                value_to_param(first)
            } else {
                QueryParam::Null
            }
        }
        _ => QueryParam::String(value.to_string()),
    }
}

/// Convert PostgreSQL row to JSON value
fn row_to_json(row: PgRow) -> Value {
    // This is a simplified version
    // In production, you'd want to properly handle column types
    serde_json::json!({
        "data": format!("{:?}", row),
    })
}

// ============================================
// Transaction Helper
// ============================================

/// Helper for database transactions
pub struct TransactionHelper {
    pool: PgPool,
}

impl TransactionHelper {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run a function within a transaction
    pub async fn run<F, R>(&self, f: F) -> Result<R>
    where
        F: for<'a> FnOnce(
                &'a mut sqlx::Transaction<'_, sqlx::Postgres>,
            ) -> futures_util::future::BoxFuture<'a, Result<R>>
            + Send
            + 'static,
        R: Send + 'static,
    {
        let mut tx = self.pool.begin().await?;

        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Run a batch of operations in a transaction
    pub async fn batch<F>(&self, operations: F) -> Result<Vec<Value>>
    where
        F: for<'a> FnOnce(
                &'a mut sqlx::Transaction<'_, sqlx::Postgres>,
            ) -> futures_util::future::BoxFuture<'a, Result<Vec<Value>>>
            + Send
            + 'static,
    {
        let mut tx = self.pool.begin().await?;

        match operations(&mut tx).await {
            Ok(results) => {
                tx.commit().await?;
                Ok(results)
            }
            Err(e) => {
                tx.rollback().await?;
                Err(e)
            }
        }
    }
}

// ============================================
// Pagination Types
// ============================================

/// Paginated result from a query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResult<T> {
    /// Check if there's a next page
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    /// Check if there's a previous page
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    /// Get the next page number
    pub fn next_page(&self) -> Option<u32> {
        if self.has_next() {
            Some(self.page + 1)
        } else {
            None
        }
    }

    /// Get the previous page number
    pub fn prev_page(&self) -> Option<u32> {
        if self.has_prev() {
            Some(self.page - 1)
        } else {
            None
        }
    }
}

// ============================================
// Common Database Operations
// ============================================

/// Upsert a record (insert or update if exists)
pub async fn upsert(
    pool: &PgPool,
    table: &str,
    data: Value,
    conflict_column: &str,
) -> Result<Value> {
    let query = format!(
        "INSERT INTO {} (data) VALUES ($1) \
         ON CONFLICT ({}) DO UPDATE \
         SET data = EXCLUDED.data \
         RETURNING *",
        table, conflict_column
    );

    let row = sqlx::query(&query)
        .bind(data)
        .fetch_one(pool)
        .await?;

    Ok(row_to_json(row))
}

/// Bulk insert records
pub async fn bulk_insert(
    pool: &PgPool,
    table: &str,
    records: Vec<Value>,
) -> Result<u64> {
    if records.is_empty() {
        return Ok(0);
    }

    // This is a simplified version
    // In production, you'd need to properly build the bulk insert query
    let query = format!("INSERT INTO {} (data) SELECT * FROM UNNEST($1::jsonb[])", table);

    let result = sqlx::query(&query)
        .bind(serde_json::json!(records))
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}

/// Soft delete a record (sets deleted_at timestamp)
pub async fn soft_delete(pool: &PgPool, table: &str, id: Uuid) -> Result<bool> {
    let query = format!(
        "UPDATE {} SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        table
    );

    let result = sqlx::query(&query)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get records by IDs
pub async fn get_by_ids(
    pool: &PgPool,
    table: &str,
    ids: Vec<Uuid>,
) -> Result<Vec<Value>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect();

    let query = format!(
        "SELECT * FROM {} WHERE id IN ({})",
        table,
        placeholders.join(", ")
    );

    let mut q = sqlx::query(&query);
    for id in &ids {
        q = q.bind(id);
    }

    let rows = q.fetch_all(pool).await?;
    let results: Vec<Value> = rows.into_iter().map(row_to_json).collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_result() {
        let result = PaginatedResult::<serde_json::Value> {
            data: vec![],
            total: 100,
            page: 2,
            per_page: 10,
            total_pages: 10,
        };

        assert!(result.has_next());
        assert!(result.has_prev());
        assert_eq!(result.next_page(), Some(3));
        assert_eq!(result.prev_page(), Some(1));
    }

    #[test]
    fn test_filter_logic() {
        assert_eq!(FilterLogic::And, FilterLogic::And);
        assert_eq!(FilterLogic::Or, FilterLogic::Or);
    }
}
