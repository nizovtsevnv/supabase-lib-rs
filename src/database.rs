//! Database module for Supabase REST API

use crate::{
    error::{Error, Result},
    types::{FilterOperator, JsonValue, OrderDirection, SupabaseConfig},
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, info};
use url::Url;

/// Database client for REST API operations
#[derive(Debug, Clone)]
pub struct Database {
    http_client: Arc<HttpClient>,
    config: Arc<SupabaseConfig>,
}

/// Query builder for SELECT operations
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    database: Database,
    table: String,
    columns: Option<String>,
    filters: Vec<Filter>,
    order_by: Vec<OrderBy>,
    limit: Option<u32>,
    offset: Option<u32>,
    single: bool,
    joins: Vec<Join>,
}

/// Represents a table join operation
#[derive(Debug, Clone)]
pub struct Join {
    /// Type of join (inner, left, right, full)
    pub join_type: JoinType,
    /// Foreign table name
    pub foreign_table: String,
    /// Columns to select from foreign table
    pub foreign_columns: String,
    /// Optional foreign table alias
    pub alias: Option<String>,
}

/// Types of JOIN operations supported by PostgREST
#[derive(Debug, Clone)]
pub enum JoinType {
    /// Inner join (default PostgREST behavior)
    Inner,
    /// Left join (includes null values)
    Left,
}

/// Transaction builder for batching multiple database operations
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    database: Database,
    operations: Vec<JsonValue>,
}

/// Types of transaction operations
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionOperation {
    Insert,
    Update,
    Delete,
    Select,
    Rpc,
}

/// Insert builder for INSERT operations
#[derive(Debug, Clone)]
pub struct InsertBuilder {
    database: Database,
    table: String,
    data: JsonValue,
    upsert: bool,
    on_conflict: Option<String>,
    returning: Option<String>,
}

/// Update builder for UPDATE operations
#[derive(Debug, Clone)]
pub struct UpdateBuilder {
    database: Database,
    table: String,
    data: JsonValue,
    filters: Vec<Filter>,
    returning: Option<String>,
}

/// Delete builder for DELETE operations
#[derive(Debug, Clone)]
pub struct DeleteBuilder {
    database: Database,
    table: String,
    filters: Vec<Filter>,
    returning: Option<String>,
}

/// Database filter for WHERE clauses
#[derive(Debug, Clone)]
pub enum Filter {
    /// Simple column filter
    Simple {
        column: String,
        operator: FilterOperator,
        value: String,
    },
    /// Logical AND group
    And(Vec<Filter>),
    /// Logical OR group
    Or(Vec<Filter>),
    /// Logical NOT filter
    Not(Box<Filter>),
}

/// Order by clause
#[derive(Debug, Clone)]
struct OrderBy {
    column: String,
    direction: OrderDirection,
    #[allow(dead_code)]
    nulls_first: Option<bool>,
}

/// Database response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseResponse<T> {
    pub data: T,
    pub count: Option<u64>,
}

impl Database {
    /// Create a new Database instance
    pub fn new(config: Arc<SupabaseConfig>, http_client: Arc<HttpClient>) -> Result<Self> {
        debug!("Initializing Database module");

        Ok(Self {
            http_client,
            config,
        })
    }

    /// Start a query from a table
    pub fn from(&self, table: &str) -> QueryBuilder {
        QueryBuilder::new(self.clone(), table.to_string())
    }

    /// Insert data into a table
    pub fn insert(&self, table: &str) -> InsertBuilder {
        InsertBuilder::new(self.clone(), table.to_string())
    }

    /// Upsert data into a table (insert or update if exists)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::{json, Value};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Upsert a user (insert if new, update if exists)
    /// let user: Vec<Value> = client.database()
    ///     .upsert("users")
    ///     .values(json!({
    ///         "id": 1,
    ///         "name": "John Doe",
    ///         "email": "john@example.com"
    ///     }))
    ///     .unwrap()
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn upsert(&self, table: &str) -> InsertBuilder {
        InsertBuilder::new(self.clone(), table.to_string()).upsert()
    }

    /// Bulk insert multiple records at once
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::{json, Value};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Insert multiple users at once
    /// let users: Vec<Value> = client.database()
    ///     .bulk_insert("users", vec![
    ///         json!({"name": "Alice", "email": "alice@example.com"}),
    ///         json!({"name": "Bob", "email": "bob@example.com"}),
    ///         json!({"name": "Charlie", "email": "charlie@example.com"})
    ///     ])
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bulk_insert<T>(&self, table: &str, data: Vec<JsonValue>) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!(
            "Executing BULK INSERT on table: {} with {} records",
            table,
            data.len()
        );

        let url = format!("{}/{}", self.rest_url(), table);
        let response = self
            .http_client
            .post(&url)
            .json(&data)
            .header("Prefer", "return=representation")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Bulk insert failed with status: {}", status),
            };
            return Err(Error::database(error_msg));
        }

        let result: Vec<T> = response.json().await?;
        info!("Bulk insert executed successfully on table: {}", table);
        Ok(result)
    }

    /// Bulk upsert multiple records at once
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::{json, Value};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Upsert multiple users (insert new, update existing)
    /// let users: Vec<Value> = client.database()
    ///     .bulk_upsert("users", vec![
    ///         json!({"id": 1, "name": "Alice Updated", "email": "alice@example.com"}),
    ///         json!({"id": 2, "name": "Bob", "email": "bob@example.com"}),
    ///     ])
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bulk_upsert<T>(&self, table: &str, data: Vec<JsonValue>) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!(
            "Executing BULK UPSERT on table: {} with {} records",
            table,
            data.len()
        );

        let url = format!("{}/{}", self.rest_url(), table);
        let response = self
            .http_client
            .post(&url)
            .json(&data)
            .header(
                "Prefer",
                "return=representation,resolution=merge-duplicates",
            )
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Bulk upsert failed with status: {}", status),
            };
            return Err(Error::database(error_msg));
        }

        let result: Vec<T> = response.json().await?;
        info!("Bulk upsert executed successfully on table: {}", table);
        Ok(result)
    }

    /// Execute raw SQL via stored procedure/function
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::{json, Value};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Execute raw SQL through RPC function
    /// let result: Vec<Value> = client.database()
    ///     .raw_sql("SELECT * FROM users WHERE age > $1 AND status = $2", json!([18, "active"]))
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn raw_sql<T>(&self, sql: &str, params: JsonValue) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing raw SQL: {}", sql);

        // Execute raw SQL via RPC function (requires a database function to handle raw SQL)
        let rpc_params = json!({
            "sql": sql,
            "params": params
        });

        let result = self.rpc("execute_sql", Some(rpc_params)).await?;

        // Convert JsonValue to Vec<T>
        match result {
            JsonValue::Array(arr) => {
                let mut results = Vec::with_capacity(arr.len());
                for item in arr {
                    let parsed: T = serde_json::from_value(item)?;
                    results.push(parsed);
                }
                Ok(results)
            }
            _ => {
                // If single result, wrap in array
                let parsed: T = serde_json::from_value(result)?;
                Ok(vec![parsed])
            }
        }
    }

    /// Execute a prepared statement with parameters (safe from SQL injection)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::{json, Value};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Execute prepared statement
    /// let users: Vec<Value> = client.database()
    ///     .prepared_statement("get_active_users_by_age", json!({"min_age": 18}))
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn prepared_statement<T>(
        &self,
        statement_name: &str,
        params: JsonValue,
    ) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing prepared statement: {}", statement_name);

        let result = self.rpc(statement_name, Some(params)).await?;

        // Convert JsonValue to Vec<T>
        match result {
            JsonValue::Array(arr) => {
                let mut results = Vec::with_capacity(arr.len());
                for item in arr {
                    let parsed: T = serde_json::from_value(item)?;
                    results.push(parsed);
                }
                Ok(results)
            }
            _ => {
                // If single result, wrap in array
                let parsed: T = serde_json::from_value(result)?;
                Ok(vec![parsed])
            }
        }
    }

    /// Execute a SQL query that returns a count (for analytical queries)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Get count of active users
    /// let count: u64 = client.database()
    ///     .count_query("count_active_users", json!({}))
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn count_query(&self, function_name: &str, params: JsonValue) -> Result<u64> {
        debug!("Executing count query: {}", function_name);

        let result = self.rpc(function_name, Some(params)).await?;

        // Extract count from result
        match result {
            JsonValue::Number(num) => num
                .as_u64()
                .ok_or_else(|| Error::database("Count query returned invalid number".to_string())),
            JsonValue::Object(obj) => obj.get("count").and_then(|v| v.as_u64()).ok_or_else(|| {
                Error::database("Count query result missing 'count' field".to_string())
            }),
            _ => Err(Error::database(
                "Count query returned unexpected result type".to_string(),
            )),
        }
    }

    /// Begin a database transaction (via RPC)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::{json, Value};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Execute multiple operations in a transaction
    /// let result: Vec<Value> = client.database()
    ///     .transaction(vec![
    ///         json!({
    ///             "operation": "insert",
    ///             "table": "users",
    ///             "data": {"name": "John", "email": "john@example.com"}
    ///         }),
    ///         json!({
    ///             "operation": "update",
    ///             "table": "profiles",
    ///             "data": {"user_id": 1, "bio": "Updated bio"},
    ///             "where": {"user_id": 1}
    ///         })
    ///     ])
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub async fn transaction<T>(&self, operations: Vec<JsonValue>) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing transaction with {} operations", operations.len());

        let transaction_params = json!({
            "operations": operations
        });

        let result = self
            .rpc("execute_transaction", Some(transaction_params))
            .await?;

        // Convert JsonValue to Vec<T>
        match result {
            JsonValue::Array(arr) => {
                let mut results = Vec::with_capacity(arr.len());
                for item in arr {
                    let parsed: T = serde_json::from_value(item)?;
                    results.push(parsed);
                }
                Ok(results)
            }
            _ => {
                // If single result, wrap in array
                let parsed: T = serde_json::from_value(result)?;
                Ok(vec![parsed])
            }
        }
    }

    /// Create a transaction builder for more complex operations
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::{json, Value};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Build and execute a transaction
    /// let result: Vec<Value> = client.database()
    ///     .begin_transaction()
    ///     .insert("users", json!({"name": "Alice", "email": "alice@example.com"}))
    ///     .update("profiles", json!({"bio": "New bio"}), "user_id = 1")
    ///     .delete("old_data", "created_at < '2023-01-01'")
    ///     .commit()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn begin_transaction(&self) -> TransactionBuilder {
        TransactionBuilder::new(self.clone())
    }

    /// Update data in a table
    pub fn update(&self, table: &str) -> UpdateBuilder {
        UpdateBuilder::new(self.clone(), table.to_string())
    }

    /// Delete data from a table
    pub fn delete(&self, table: &str) -> DeleteBuilder {
        DeleteBuilder::new(self.clone(), table.to_string())
    }

    /// Execute a custom SQL query via RPC
    pub async fn rpc(&self, function_name: &str, params: Option<JsonValue>) -> Result<JsonValue> {
        debug!("Executing RPC function: {}", function_name);

        let url = format!("{}/rest/v1/rpc/{}", self.config.url, function_name);

        let mut request = self.http_client.post(&url);

        if let Some(params) = params {
            request = request.json(&params);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("RPC failed with status: {}", status),
            };
            return Err(Error::database(error_msg));
        }

        let result: JsonValue = response.json().await?;
        info!("RPC function {} executed successfully", function_name);

        Ok(result)
    }

    /// Get the base REST URL
    fn rest_url(&self) -> String {
        format!("{}/rest/v1", self.config.url)
    }

    /// Build query parameters from filters
    fn build_query_params(&self, filters: &[Filter]) -> HashMap<String, String> {
        let mut params = HashMap::new();

        for filter in filters {
            self.build_filter_params(filter, &mut params);
        }

        params
    }

    /// Build parameters for a single filter (recursive for logical operators)
    fn build_filter_params(&self, filter: &Filter, params: &mut HashMap<String, String>) {
        match filter {
            Filter::Simple {
                column,
                operator,
                value,
            } => {
                let filter_value = match operator {
                    FilterOperator::Equal => format!("eq.{}", value),
                    FilterOperator::NotEqual => format!("neq.{}", value),
                    FilterOperator::GreaterThan => format!("gt.{}", value),
                    FilterOperator::GreaterThanOrEqual => format!("gte.{}", value),
                    FilterOperator::LessThan => format!("lt.{}", value),
                    FilterOperator::LessThanOrEqual => format!("lte.{}", value),
                    FilterOperator::Like => format!("like.{}", value),
                    FilterOperator::ILike => format!("ilike.{}", value),
                    FilterOperator::Is => format!("is.{}", value),
                    FilterOperator::In => format!("in.{}", value),
                    FilterOperator::Contains => format!("cs.{}", value),
                    FilterOperator::ContainedBy => format!("cd.{}", value),
                    FilterOperator::StrictlyLeft => format!("sl.{}", value),
                    FilterOperator::StrictlyRight => format!("sr.{}", value),
                    FilterOperator::NotExtendToRight => format!("nxr.{}", value),
                    FilterOperator::NotExtendToLeft => format!("nxl.{}", value),
                    FilterOperator::Adjacent => format!("adj.{}", value),
                };

                params.insert(column.clone(), filter_value);
            }
            Filter::And(filters) => {
                // AND is the default behavior in PostgREST - just add all filters
                for filter in filters {
                    self.build_filter_params(filter, params);
                }
            }
            Filter::Or(filters) => {
                // OR requires special syntax: or=(condition1,condition2,...)
                let or_conditions: Vec<String> = filters
                    .iter()
                    .map(|f| self.build_filter_condition(f))
                    .collect();

                if !or_conditions.is_empty() {
                    let or_value = format!("({})", or_conditions.join(","));
                    params.insert("or".to_string(), or_value);
                }
            }
            Filter::Not(filter) => {
                // NOT requires prefixing with "not."
                match filter.as_ref() {
                    Filter::Simple {
                        column,
                        operator,
                        value,
                    } => {
                        let filter_value = match operator {
                            FilterOperator::Equal => format!("eq.{}", value),
                            FilterOperator::NotEqual => format!("neq.{}", value),
                            FilterOperator::GreaterThan => format!("gt.{}", value),
                            FilterOperator::GreaterThanOrEqual => format!("gte.{}", value),
                            FilterOperator::LessThan => format!("lt.{}", value),
                            FilterOperator::LessThanOrEqual => format!("lte.{}", value),
                            FilterOperator::Like => format!("like.{}", value),
                            FilterOperator::ILike => format!("ilike.{}", value),
                            FilterOperator::Is => format!("is.{}", value),
                            FilterOperator::In => format!("in.{}", value),
                            FilterOperator::Contains => format!("cs.{}", value),
                            FilterOperator::ContainedBy => format!("cd.{}", value),
                            FilterOperator::StrictlyLeft => format!("sl.{}", value),
                            FilterOperator::StrictlyRight => format!("sr.{}", value),
                            FilterOperator::NotExtendToRight => format!("nxr.{}", value),
                            FilterOperator::NotExtendToLeft => format!("nxl.{}", value),
                            FilterOperator::Adjacent => format!("adj.{}", value),
                        };

                        params.insert(format!("not.{}", column), filter_value);
                    }
                    Filter::And(and_filters) => {
                        // NOT(AND(...)) becomes NOT with multiple conditions
                        let and_conditions: Vec<String> = and_filters
                            .iter()
                            .map(|f| self.build_filter_condition(f))
                            .collect();

                        if !and_conditions.is_empty() {
                            let not_value = format!("and.({})", and_conditions.join(","));
                            params.insert("not".to_string(), not_value);
                        }
                    }
                    Filter::Or(or_filters) => {
                        // NOT(OR(...)) becomes NOT with OR conditions
                        let or_conditions: Vec<String> = or_filters
                            .iter()
                            .map(|f| self.build_filter_condition(f))
                            .collect();

                        if !or_conditions.is_empty() {
                            let not_value = format!("or.({})", or_conditions.join(","));
                            params.insert("not".to_string(), not_value);
                        }
                    }
                    Filter::Not(_) => {
                        // Double negation - just apply the inner filter normally
                        // NOT(NOT(x)) = x
                        if let Filter::Not(inner) = filter.as_ref() {
                            self.build_filter_params(inner, params);
                        }
                    }
                }
            }
        }
    }

    /// Build a single condition string for complex filters
    #[allow(clippy::only_used_in_recursion)]
    fn build_filter_condition(&self, filter: &Filter) -> String {
        match filter {
            Filter::Simple {
                column,
                operator,
                value,
            } => {
                let op_str = match operator {
                    FilterOperator::Equal => "eq",
                    FilterOperator::NotEqual => "neq",
                    FilterOperator::GreaterThan => "gt",
                    FilterOperator::GreaterThanOrEqual => "gte",
                    FilterOperator::LessThan => "lt",
                    FilterOperator::LessThanOrEqual => "lte",
                    FilterOperator::Like => "like",
                    FilterOperator::ILike => "ilike",
                    FilterOperator::Is => "is",
                    FilterOperator::In => "in",
                    FilterOperator::Contains => "cs",
                    FilterOperator::ContainedBy => "cd",
                    FilterOperator::StrictlyLeft => "sl",
                    FilterOperator::StrictlyRight => "sr",
                    FilterOperator::NotExtendToRight => "nxr",
                    FilterOperator::NotExtendToLeft => "nxl",
                    FilterOperator::Adjacent => "adj",
                };
                format!("{}.{}.{}", column, op_str, value)
            }
            Filter::And(filters) => {
                let conditions: Vec<String> = filters
                    .iter()
                    .map(|f| self.build_filter_condition(f))
                    .collect();
                format!("and.({})", conditions.join(","))
            }
            Filter::Or(filters) => {
                let conditions: Vec<String> = filters
                    .iter()
                    .map(|f| self.build_filter_condition(f))
                    .collect();
                format!("or.({})", conditions.join(","))
            }
            Filter::Not(filter) => {
                let condition = self.build_filter_condition(filter);
                format!("not.({})", condition)
            }
        }
    }
}

impl QueryBuilder {
    fn new(database: Database, table: String) -> Self {
        Self {
            database,
            table,
            columns: None,
            filters: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            single: false,
            joins: Vec::new(),
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: &str) -> Self {
        self.columns = Some(columns.to_string());
        self
    }

    /// Add an equality filter
    pub fn eq(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::Equal,
            value: value.to_string(),
        });
        self
    }

    /// Add a not equal filter
    pub fn neq(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::NotEqual,
            value: value.to_string(),
        });
        self
    }

    /// Add a greater than filter
    pub fn gt(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::GreaterThan,
            value: value.to_string(),
        });
        self
    }

    /// Add a greater than or equal filter
    pub fn gte(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::GreaterThanOrEqual,
            value: value.to_string(),
        });
        self
    }

    /// Add a less than filter
    pub fn lt(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::LessThan,
            value: value.to_string(),
        });
        self
    }

    /// Add a less than or equal filter
    pub fn lte(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::LessThanOrEqual,
            value: value.to_string(),
        });
        self
    }

    /// Add a LIKE filter
    pub fn like(mut self, column: &str, pattern: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::Like,
            value: pattern.to_string(),
        });
        self
    }

    /// Add an ILIKE filter (case-insensitive)
    pub fn ilike(mut self, column: &str, pattern: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::ILike,
            value: pattern.to_string(),
        });
        self
    }

    /// Add an IS filter (for null checks)
    pub fn is(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::Is,
            value: value.to_string(),
        });
        self
    }

    /// Add an IN filter
    pub fn r#in(mut self, column: &str, values: &[&str]) -> Self {
        let value = format!("({})", values.join(","));
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::In,
            value,
        });
        self
    }

    /// Add ordering
    pub fn order(mut self, column: &str, direction: OrderDirection) -> Self {
        self.order_by.push(OrderBy {
            column: column.to_string(),
            direction,
            nulls_first: None,
        });
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Return single row
    pub fn single(mut self) -> Self {
        self.single = true;
        self
    }

    /// Group filters with AND logic
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::Value;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Find users where age >= 18 AND status = "active" AND city = "NYC"
    /// let adults_in_nyc: Vec<Value> = client.database()
    ///     .from("users")
    ///     .select("*")
    ///     .and(|query| {
    ///         query
    ///             .gte("age", "18")
    ///             .eq("status", "active")
    ///             .eq("city", "NYC")
    ///     })
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn and<F>(mut self, builder_fn: F) -> Self
    where
        F: FnOnce(QueryBuilder) -> QueryBuilder,
    {
        // Create a new query builder for the AND group
        let and_builder = QueryBuilder::new(self.database.clone(), self.table.clone());
        let built_query = builder_fn(and_builder);

        if !built_query.filters.is_empty() {
            self.filters.push(Filter::And(built_query.filters));
        }

        self
    }

    /// Group filters with OR logic
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::Value;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Find users where status = "online" OR status = "away"
    /// let active_users: Vec<Value> = client.database()
    ///     .from("users")
    ///     .select("*")
    ///     .or(|query| {
    ///         query
    ///             .eq("status", "online")
    ///             .eq("status", "away")
    ///     })
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn or<F>(mut self, builder_fn: F) -> Self
    where
        F: FnOnce(QueryBuilder) -> QueryBuilder,
    {
        // Create a new query builder for the OR group
        let or_builder = QueryBuilder::new(self.database.clone(), self.table.clone());
        let built_query = builder_fn(or_builder);

        if !built_query.filters.is_empty() {
            self.filters.push(Filter::Or(built_query.filters));
        }

        self
    }

    /// Apply NOT logic to a filter
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::Value;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Find users where NOT (status = "banned")
    /// let active_users: Vec<Value> = client.database()
    ///     .from("users")
    ///     .select("*")
    ///     .not(|query| query.eq("status", "banned"))
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn not<F>(mut self, builder_fn: F) -> Self
    where
        F: FnOnce(QueryBuilder) -> QueryBuilder,
    {
        // Create a new query builder for the NOT condition
        let not_builder = QueryBuilder::new(self.database.clone(), self.table.clone());
        let built_query = builder_fn(not_builder);

        if !built_query.filters.is_empty() {
            // Wrap all filters in a single NOT
            if built_query.filters.len() == 1 {
                self.filters
                    .push(Filter::Not(Box::new(built_query.filters[0].clone())));
            } else {
                // Multiple filters get wrapped in AND, then NOT
                self.filters
                    .push(Filter::Not(Box::new(Filter::And(built_query.filters))));
            }
        }

        self
    }

    /// Add an INNER JOIN to another table
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::Value;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Join posts with authors: SELECT posts.*, authors.name, authors.email
    /// let posts_with_authors: Vec<Value> = client.database()
    ///     .from("posts")
    ///     .select("*")
    ///     .inner_join("authors", "name,email")
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn inner_join(mut self, foreign_table: &str, foreign_columns: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            foreign_table: foreign_table.to_string(),
            foreign_columns: foreign_columns.to_string(),
            alias: None,
        });
        self
    }

    /// Add a LEFT JOIN to another table
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::Value;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Left join posts with optional authors
    /// let posts_with_optional_authors: Vec<Value> = client.database()
    ///     .from("posts")
    ///     .select("*")
    ///     .left_join("authors", "name,email")
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn left_join(mut self, foreign_table: &str, foreign_columns: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Left,
            foreign_table: foreign_table.to_string(),
            foreign_columns: foreign_columns.to_string(),
            alias: None,
        });
        self
    }

    /// Add an INNER JOIN with a custom alias
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::Value;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Join with alias: SELECT posts.*, author:authors!inner(name,email)
    /// let posts: Vec<Value> = client.database()
    ///     .from("posts")
    ///     .select("*")
    ///     .inner_join_as("authors", "name,email", "author")
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn inner_join_as(
        mut self,
        foreign_table: &str,
        foreign_columns: &str,
        alias: &str,
    ) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Inner,
            foreign_table: foreign_table.to_string(),
            foreign_columns: foreign_columns.to_string(),
            alias: Some(alias.to_string()),
        });
        self
    }

    /// Add a LEFT JOIN with a custom alias
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use supabase_lib_rs::Client;
    /// # use serde_json::Value;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://localhost:54321", "test-key").unwrap();
    ///
    /// // Left join with alias: SELECT posts.*, author:authors(name,email)
    /// let posts: Vec<Value> = client.database()
    ///     .from("posts")
    ///     .select("*")
    ///     .left_join_as("authors", "name,email", "author")
    ///     .execute()
    ///     .await
    ///     .unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn left_join_as(mut self, foreign_table: &str, foreign_columns: &str, alias: &str) -> Self {
        self.joins.push(Join {
            join_type: JoinType::Left,
            foreign_table: foreign_table.to_string(),
            foreign_columns: foreign_columns.to_string(),
            alias: Some(alias.to_string()),
        });
        self
    }

    /// Execute the query
    pub async fn execute<T>(&self) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing SELECT query on table: {}", self.table);

        let mut url = Url::parse(&format!("{}/{}", self.database.rest_url(), self.table))?;

        // Add query parameters
        let mut query_params = self.database.build_query_params(&self.filters);

        // Build select statement with joins
        let select_clause = self.build_select_with_joins();
        query_params.insert("select".to_string(), select_clause);

        if !self.order_by.is_empty() {
            let order_clauses: Vec<String> = self
                .order_by
                .iter()
                .map(|order| {
                    let direction = match order.direction {
                        OrderDirection::Ascending => "asc",
                        OrderDirection::Descending => "desc",
                    };
                    format!("{}.{}", order.column, direction)
                })
                .collect();
            query_params.insert("order".to_string(), order_clauses.join(","));
        }

        if let Some(limit) = self.limit {
            query_params.insert("limit".to_string(), limit.to_string());
        }

        if let Some(offset) = self.offset {
            query_params.insert("offset".to_string(), offset.to_string());
        }

        // Set URL query parameters
        for (key, value) in query_params {
            url.query_pairs_mut().append_pair(&key, &value);
        }

        debug!("Generated query URL: {}", url.as_str());
        let mut request = self.database.http_client.get(url.as_str());

        if self.single {
            request = request.header("Accept", "application/vnd.pgrst.object+json");
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Query failed with status: {}", status),
            };
            return Err(Error::database(error_msg));
        }

        let result = if self.single {
            let single_item: T = response.json().await?;
            vec![single_item]
        } else {
            response.json().await?
        };

        info!(
            "SELECT query executed successfully on table: {}",
            self.table
        );
        Ok(result)
    }

    /// Build the SELECT clause including any joins
    fn build_select_with_joins(&self) -> String {
        let base_columns = self.columns.as_deref().unwrap_or("*");

        if self.joins.is_empty() {
            return base_columns.to_string();
        }

        let mut select_parts = vec![base_columns.to_string()];

        for join in &self.joins {
            let join_clause = self.build_join_clause(join);
            select_parts.push(join_clause);
        }

        select_parts.join(",")
    }

    /// Build a single join clause for PostgREST
    fn build_join_clause(&self, join: &Join) -> String {
        match (&join.alias, &join.join_type) {
            (Some(alias), JoinType::Inner) => {
                // alias:foreign_table!inner(columns)
                format!(
                    "{}:{}!inner({})",
                    alias, join.foreign_table, join.foreign_columns
                )
            }
            (Some(alias), JoinType::Left) => {
                // alias:foreign_table(columns)
                format!("{}:{}({})", alias, join.foreign_table, join.foreign_columns)
            }
            (None, JoinType::Inner) => {
                // foreign_table!inner(columns)
                format!("{}!inner({})", join.foreign_table, join.foreign_columns)
            }
            (None, JoinType::Left) => {
                // foreign_table(columns)
                format!("{}({})", join.foreign_table, join.foreign_columns)
            }
        }
    }

    /// Execute the query and return a single row
    pub async fn single_execute<T>(&self) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut builder = self.clone();
        builder.single = true;

        let results = builder.execute().await?;
        Ok(results.into_iter().next())
    }
}

impl InsertBuilder {
    fn new(database: Database, table: String) -> Self {
        Self {
            database,
            table,
            data: JsonValue::Null,
            upsert: false,
            on_conflict: None,
            returning: None,
        }
    }

    /// Set the data to insert
    pub fn values<T: Serialize>(mut self, data: T) -> Result<Self> {
        self.data = serde_json::to_value(data)?;
        Ok(self)
    }

    /// Enable upsert mode
    pub fn upsert(mut self) -> Self {
        self.upsert = true;
        self
    }

    /// Set conflict resolution
    pub fn on_conflict(mut self, columns: &str) -> Self {
        self.on_conflict = Some(columns.to_string());
        self
    }

    /// Set columns to return
    pub fn returning(mut self, columns: &str) -> Self {
        self.returning = Some(columns.to_string());
        self
    }

    /// Execute the insert
    pub async fn execute<T>(&self) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing INSERT query on table: {}", self.table);

        let url = format!("{}/{}", self.database.rest_url(), self.table);
        let mut request = self.database.http_client.post(&url).json(&self.data);

        if let Some(ref _returning) = self.returning {
            request = request.header("Prefer", "return=representation".to_string());
        }

        if self.upsert {
            request = request.header("Prefer", "resolution=merge-duplicates");
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Insert failed with status: {}", status),
            };
            return Err(Error::database(error_msg));
        }

        let result: Vec<T> = response.json().await?;
        info!(
            "INSERT query executed successfully on table: {}",
            self.table
        );

        Ok(result)
    }
}

impl UpdateBuilder {
    fn new(database: Database, table: String) -> Self {
        Self {
            database,
            table,
            data: JsonValue::Null,
            filters: Vec::new(),
            returning: None,
        }
    }

    /// Set the data to update
    pub fn set<T: Serialize>(mut self, data: T) -> Result<Self> {
        self.data = serde_json::to_value(data)?;
        Ok(self)
    }

    /// Add an equality filter
    pub fn eq(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::Equal,
            value: value.to_string(),
        });
        self
    }

    /// Set columns to return
    pub fn returning(mut self, columns: &str) -> Self {
        self.returning = Some(columns.to_string());
        self
    }

    /// Execute the update
    pub async fn execute<T>(&self) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing UPDATE query on table: {}", self.table);

        let mut url = Url::parse(&format!("{}/{}", self.database.rest_url(), self.table))?;

        // Add filters as query parameters
        let query_params = self.database.build_query_params(&self.filters);
        for (key, value) in query_params {
            url.query_pairs_mut().append_pair(&key, &value);
        }

        let mut request = self
            .database
            .http_client
            .patch(url.as_str())
            .json(&self.data);

        if let Some(ref _returning) = self.returning {
            request = request.header("Prefer", "return=representation");
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Update failed with status: {}", status),
            };
            return Err(Error::database(error_msg));
        }

        let result: Vec<T> = response.json().await?;
        info!(
            "UPDATE query executed successfully on table: {}",
            self.table
        );

        Ok(result)
    }
}

impl DeleteBuilder {
    fn new(database: Database, table: String) -> Self {
        Self {
            database,
            table,
            filters: Vec::new(),
            returning: None,
        }
    }

    /// Add an equality filter
    pub fn eq(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter::Simple {
            column: column.to_string(),
            operator: FilterOperator::Equal,
            value: value.to_string(),
        });
        self
    }

    /// Set columns to return
    pub fn returning(mut self, columns: &str) -> Self {
        self.returning = Some(columns.to_string());
        self
    }

    /// Execute the delete
    pub async fn execute<T>(&self) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing DELETE query on table: {}", self.table);

        let mut url = Url::parse(&format!("{}/{}", self.database.rest_url(), self.table))?;

        // Add filters as query parameters
        let query_params = self.database.build_query_params(&self.filters);
        for (key, value) in query_params {
            url.query_pairs_mut().append_pair(&key, &value);
        }

        let mut request = self.database.http_client.delete(url.as_str());

        if let Some(ref _returning) = self.returning {
            request = request.header("Prefer", "return=representation");
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_msg = match response.text().await {
                Ok(text) => text,
                Err(_) => format!("Delete failed with status: {}", status),
            };
            return Err(Error::database(error_msg));
        }

        let result: Vec<T> = response.json().await?;
        info!(
            "DELETE query executed successfully on table: {}",
            self.table
        );

        Ok(result)
    }
}

impl TransactionBuilder {
    fn new(database: Database) -> Self {
        Self {
            database,
            operations: Vec::new(),
        }
    }

    /// Add an INSERT operation to the transaction
    pub fn insert(mut self, table: &str, data: JsonValue) -> Self {
        self.operations.push(json!({
            "operation": TransactionOperation::Insert,
            "table": table,
            "data": data
        }));
        self
    }

    /// Add an UPDATE operation to the transaction
    pub fn update(mut self, table: &str, data: JsonValue, where_clause: &str) -> Self {
        self.operations.push(json!({
            "operation": TransactionOperation::Update,
            "table": table,
            "data": data,
            "where": where_clause
        }));
        self
    }

    /// Add a DELETE operation to the transaction
    pub fn delete(mut self, table: &str, where_clause: &str) -> Self {
        self.operations.push(json!({
            "operation": TransactionOperation::Delete,
            "table": table,
            "where": where_clause
        }));
        self
    }

    /// Add a SELECT operation to the transaction
    pub fn select(mut self, table: &str, columns: &str, where_clause: Option<&str>) -> Self {
        let mut operation = json!({
            "operation": TransactionOperation::Select,
            "table": table,
            "columns": columns
        });

        if let Some(where_clause) = where_clause {
            operation["where"] = json!(where_clause);
        }

        self.operations.push(operation);
        self
    }

    /// Add an RPC call to the transaction
    pub fn rpc(mut self, function_name: &str, params: JsonValue) -> Self {
        self.operations.push(json!({
            "operation": TransactionOperation::Rpc,
            "function": function_name,
            "params": params
        }));
        self
    }

    /// Commit the transaction and execute all operations
    pub async fn commit<T>(self) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if self.operations.is_empty() {
            return Ok(Vec::new());
        }

        self.database.transaction(self.operations).await
    }

    /// Get the number of operations in the transaction
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if the transaction is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_operators() {
        // Test AND operator
        let and_filter = Filter::And(vec![
            Filter::Simple {
                column: "age".to_string(),
                operator: FilterOperator::GreaterThanOrEqual,
                value: "18".to_string(),
            },
            Filter::Simple {
                column: "status".to_string(),
                operator: FilterOperator::Equal,
                value: "active".to_string(),
            },
        ]);

        // Test OR operator
        let or_filter = Filter::Or(vec![
            Filter::Simple {
                column: "role".to_string(),
                operator: FilterOperator::Equal,
                value: "admin".to_string(),
            },
            Filter::Simple {
                column: "role".to_string(),
                operator: FilterOperator::Equal,
                value: "moderator".to_string(),
            },
        ]);

        // Test NOT operator
        let not_filter = Filter::Not(Box::new(Filter::Simple {
            column: "status".to_string(),
            operator: FilterOperator::Equal,
            value: "banned".to_string(),
        }));

        // These should compile and be constructed properly
        assert!(matches!(and_filter, Filter::And(_)));
        assert!(matches!(or_filter, Filter::Or(_)));
        assert!(matches!(not_filter, Filter::Not(_)));
    }

    #[test]
    fn test_filter_condition_generation() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        // Test simple condition
        let simple_filter = Filter::Simple {
            column: "age".to_string(),
            operator: FilterOperator::Equal,
            value: "25".to_string(),
        };
        let condition = db.build_filter_condition(&simple_filter);
        assert_eq!(condition, "age.eq.25");

        // Test OR condition
        let or_filter = Filter::Or(vec![
            Filter::Simple {
                column: "status".to_string(),
                operator: FilterOperator::Equal,
                value: "online".to_string(),
            },
            Filter::Simple {
                column: "status".to_string(),
                operator: FilterOperator::Equal,
                value: "away".to_string(),
            },
        ]);
        let condition = db.build_filter_condition(&or_filter);
        assert_eq!(condition, "or.(status.eq.online,status.eq.away)");

        // Test NOT condition
        let not_filter = Filter::Not(Box::new(Filter::Simple {
            column: "banned".to_string(),
            operator: FilterOperator::Equal,
            value: "true".to_string(),
        }));
        let condition = db.build_filter_condition(&not_filter);
        assert_eq!(condition, "not.(banned.eq.true)");
    }

    #[test]
    fn test_query_builder_logical_methods() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        // Test AND builder
        let query = db
            .from("users")
            .select("*")
            .and(|q| q.gte("age", "18").eq("status", "active"));

        // Should have one AND filter
        assert_eq!(query.filters.len(), 1);
        assert!(matches!(query.filters[0], Filter::And(_)));

        // Test OR builder
        let query = db
            .from("users")
            .select("*")
            .or(|q| q.eq("role", "admin").eq("role", "mod"));

        // Should have one OR filter
        assert_eq!(query.filters.len(), 1);
        assert!(matches!(query.filters[0], Filter::Or(_)));

        // Test NOT builder
        let query = db.from("users").select("*").not(|q| q.eq("banned", "true"));

        // Should have one NOT filter
        assert_eq!(query.filters.len(), 1);
        assert!(matches!(query.filters[0], Filter::Not(_)));
    }

    #[test]
    fn test_join_functionality() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        // Test inner join
        let query = db
            .from("posts")
            .select("title,content")
            .inner_join("authors", "name,email");

        assert_eq!(query.joins.len(), 1);
        let join = &query.joins[0];
        assert!(matches!(join.join_type, JoinType::Inner));
        assert_eq!(join.foreign_table, "authors");
        assert_eq!(join.foreign_columns, "name,email");
        assert!(join.alias.is_none());

        // Test left join with alias
        let query = db
            .from("posts")
            .select("*")
            .left_join_as("authors", "name", "author");

        assert_eq!(query.joins.len(), 1);
        let join = &query.joins[0];
        assert!(matches!(join.join_type, JoinType::Left));
        assert_eq!(join.foreign_table, "authors");
        assert_eq!(join.foreign_columns, "name");
        assert_eq!(join.alias.as_ref().unwrap(), "author");
    }

    #[test]
    fn test_join_clause_generation() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();
        let query = QueryBuilder::new(db, "posts".to_string());

        // Test inner join without alias
        let inner_join = Join {
            join_type: JoinType::Inner,
            foreign_table: "authors".to_string(),
            foreign_columns: "name,email".to_string(),
            alias: None,
        };
        let clause = query.build_join_clause(&inner_join);
        assert_eq!(clause, "authors!inner(name,email)");

        // Test left join with alias
        let left_join = Join {
            join_type: JoinType::Left,
            foreign_table: "authors".to_string(),
            foreign_columns: "name".to_string(),
            alias: Some("author".to_string()),
        };
        let clause = query.build_join_clause(&left_join);
        assert_eq!(clause, "author:authors(name)");

        // Test inner join with alias
        let inner_join_alias = Join {
            join_type: JoinType::Inner,
            foreign_table: "categories".to_string(),
            foreign_columns: "name,description".to_string(),
            alias: Some("category".to_string()),
        };
        let clause = query.build_join_clause(&inner_join_alias);
        assert_eq!(clause, "category:categories!inner(name,description)");
    }

    #[test]
    fn test_select_with_joins() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        // Test select with multiple joins
        let query = db
            .from("posts")
            .select("title,content")
            .inner_join("authors", "name,email")
            .left_join_as("categories", "name", "category");

        let select_clause = query.build_select_with_joins();
        assert_eq!(
            select_clause,
            "title,content,authors!inner(name,email),category:categories(name)"
        );

        // Test select with default columns
        let query = db.from("posts").inner_join("authors", "name");

        let select_clause = query.build_select_with_joins();
        assert_eq!(select_clause, "*,authors!inner(name)");
    }

    #[test]
    fn test_upsert_functionality() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        // Test upsert builder
        let builder = db.upsert("users");
        assert!(builder.upsert);
        assert!(builder.on_conflict.is_none());

        // Test on_conflict
        let builder = db.insert("users").upsert().on_conflict("id");
        assert!(builder.upsert);
        assert_eq!(builder.on_conflict.as_ref().unwrap(), "id");
    }

    #[test]
    fn test_insert_builder_methods() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        // Test basic insert builder
        let builder = db.insert("users");
        assert!(!builder.upsert);
        assert!(builder.on_conflict.is_none());

        // Test upsert mode
        let builder = db.insert("users").upsert();
        assert!(builder.upsert);

        // Test on_conflict setting
        let builder = db.insert("users").upsert().on_conflict("email,id");
        assert!(builder.upsert);
        assert_eq!(builder.on_conflict.as_ref().unwrap(), "email,id");
    }

    #[test]
    fn test_transaction_builder() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        // Test empty transaction
        let tx = db.begin_transaction();
        assert!(tx.is_empty());
        assert_eq!(tx.len(), 0);

        // Test transaction with operations
        let tx = db
            .begin_transaction()
            .insert("users", json!({"name": "Alice"}))
            .update("profiles", json!({"bio": "Updated"}), "user_id = 1")
            .delete("logs", "created_at < '2023-01-01'")
            .select("settings", "*", Some("user_id = 1"))
            .rpc("calculate_stats", json!({"param": "value"}));

        assert!(!tx.is_empty());
        assert_eq!(tx.len(), 5);

        // Check operation types
        assert_eq!(tx.operations[0]["operation"], "insert");
        assert_eq!(tx.operations[1]["operation"], "update");
        assert_eq!(tx.operations[2]["operation"], "delete");
        assert_eq!(tx.operations[3]["operation"], "select");
        assert_eq!(tx.operations[4]["operation"], "rpc");
    }

    #[test]
    fn test_transaction_operation_data() {
        use crate::types::SupabaseConfig;
        use reqwest::Client as HttpClient;
        use std::sync::Arc;

        let config = Arc::new(SupabaseConfig::default());
        let http_client = Arc::new(HttpClient::new());
        let db = Database::new(config, http_client).unwrap();

        let tx = db
            .begin_transaction()
            .insert("users", json!({"name": "Bob", "email": "bob@example.com"}))
            .update("users", json!({"status": "active"}), "id = 1");

        // Test insert operation data
        let insert_op = &tx.operations[0];
        assert_eq!(insert_op["table"], "users");
        assert_eq!(insert_op["data"]["name"], "Bob");
        assert_eq!(insert_op["data"]["email"], "bob@example.com");

        // Test update operation data
        let update_op = &tx.operations[1];
        assert_eq!(update_op["table"], "users");
        assert_eq!(update_op["data"]["status"], "active");
        assert_eq!(update_op["where"], "id = 1");
    }
}
