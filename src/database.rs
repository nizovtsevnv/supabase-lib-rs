//! Database module for Supabase REST API

use crate::{
    error::{Error, Result},
    types::{FilterOperator, JsonValue, OrderDirection, SupabaseConfig},
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
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
struct Filter {
    column: String,
    operator: FilterOperator,
    value: String,
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
            let (key, value) = match filter.operator {
                FilterOperator::Equal => (filter.column.clone(), format!("eq.{}", filter.value)),
                FilterOperator::NotEqual => {
                    (filter.column.clone(), format!("neq.{}", filter.value))
                }
                FilterOperator::GreaterThan => {
                    (filter.column.clone(), format!("gt.{}", filter.value))
                }
                FilterOperator::GreaterThanOrEqual => {
                    (filter.column.clone(), format!("gte.{}", filter.value))
                }
                FilterOperator::LessThan => (filter.column.clone(), format!("lt.{}", filter.value)),
                FilterOperator::LessThanOrEqual => {
                    (filter.column.clone(), format!("lte.{}", filter.value))
                }
                FilterOperator::Like => (filter.column.clone(), format!("like.{}", filter.value)),
                FilterOperator::ILike => (filter.column.clone(), format!("ilike.{}", filter.value)),
                FilterOperator::Is => (filter.column.clone(), format!("is.{}", filter.value)),
                FilterOperator::In => (filter.column.clone(), format!("in.{}", filter.value)),
                FilterOperator::Contains => (filter.column.clone(), format!("cs.{}", filter.value)),
                FilterOperator::ContainedBy => {
                    (filter.column.clone(), format!("cd.{}", filter.value))
                }
                FilterOperator::StrictlyLeft => {
                    (filter.column.clone(), format!("sl.{}", filter.value))
                }
                FilterOperator::StrictlyRight => {
                    (filter.column.clone(), format!("sr.{}", filter.value))
                }
                FilterOperator::NotExtendToRight => {
                    (filter.column.clone(), format!("nxr.{}", filter.value))
                }
                FilterOperator::NotExtendToLeft => {
                    (filter.column.clone(), format!("nxl.{}", filter.value))
                }
                FilterOperator::Adjacent => {
                    (filter.column.clone(), format!("adj.{}", filter.value))
                }
            };

            params.insert(key, value);
        }

        params
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
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: &str) -> Self {
        self.columns = Some(columns.to_string());
        self
    }

    /// Add an equality filter
    pub fn eq(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::Equal,
            value: value.to_string(),
        });
        self
    }

    /// Add a not equal filter
    pub fn neq(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::NotEqual,
            value: value.to_string(),
        });
        self
    }

    /// Add a greater than filter
    pub fn gt(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::GreaterThan,
            value: value.to_string(),
        });
        self
    }

    /// Add a greater than or equal filter
    pub fn gte(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::GreaterThanOrEqual,
            value: value.to_string(),
        });
        self
    }

    /// Add a less than filter
    pub fn lt(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::LessThan,
            value: value.to_string(),
        });
        self
    }

    /// Add a less than or equal filter
    pub fn lte(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::LessThanOrEqual,
            value: value.to_string(),
        });
        self
    }

    /// Add a LIKE filter
    pub fn like(mut self, column: &str, pattern: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::Like,
            value: pattern.to_string(),
        });
        self
    }

    /// Add an ILIKE filter (case-insensitive)
    pub fn ilike(mut self, column: &str, pattern: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::ILike,
            value: pattern.to_string(),
        });
        self
    }

    /// Add an IS filter (for null checks)
    pub fn is(mut self, column: &str, value: &str) -> Self {
        self.filters.push(Filter {
            column: column.to_string(),
            operator: FilterOperator::Is,
            value: value.to_string(),
        });
        self
    }

    /// Add an IN filter
    pub fn r#in(mut self, column: &str, values: &[&str]) -> Self {
        let value = format!("({})", values.join(","));
        self.filters.push(Filter {
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

    /// Execute the query
    pub async fn execute<T>(&self) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Executing SELECT query on table: {}", self.table);

        let mut url = Url::parse(&format!("{}/{}", self.database.rest_url(), self.table))?;

        // Add query parameters
        let mut query_params = self.database.build_query_params(&self.filters);

        if let Some(ref columns) = self.columns {
            query_params.insert("select".to_string(), columns.clone());
        }

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
        self.filters.push(Filter {
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
        self.filters.push(Filter {
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
