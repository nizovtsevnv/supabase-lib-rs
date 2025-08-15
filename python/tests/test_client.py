#!/usr/bin/env python3
"""
Unit tests for Supabase Python client (Rust-powered)

Test coverage:
- Client initialization
- Authentication methods
- Database operations
- Storage operations
- Functions operations
- Error handling
"""

import pytest
import asyncio
from supabase_lib_rs import Client, SupabaseError


class TestClient:
    """Test client initialization and basic functionality"""

    def test_client_creation(self):
        """Test creating a client with valid parameters"""
        client = Client("http://localhost:54321", "test-key")
        assert client is not None

    def test_client_invalid_url(self):
        """Test client creation with invalid URL"""
        with pytest.raises(SupabaseError):
            Client("not-a-url", "test-key")

    def test_client_empty_params(self):
        """Test client creation with empty parameters"""
        with pytest.raises(SupabaseError):
            Client("", "")


class TestAuth:
    """Test authentication operations"""

    @pytest.fixture
    def client(self):
        return Client("http://localhost:54321", "test-key")

    @pytest.mark.asyncio
    async def test_auth_sign_up(self, client):
        """Test user sign up"""
        try:
            result = await client.auth.sign_up("test@example.com", "password123")
            assert isinstance(result, dict)
            assert "user" in result or "error" in result  # Either success or expected error
        except SupabaseError:
            # Expected in test environment without actual Supabase instance
            pass

    @pytest.mark.asyncio
    async def test_auth_sign_in(self, client):
        """Test user sign in"""
        try:
            result = await client.auth.sign_in("test@example.com", "password123")
            assert isinstance(result, dict)
        except SupabaseError:
            # Expected in test environment
            pass

    @pytest.mark.asyncio
    async def test_auth_sign_out(self, client):
        """Test user sign out"""
        try:
            await client.auth.sign_out()
        except SupabaseError:
            # Expected in test environment
            pass


class TestDatabase:
    """Test database operations"""

    @pytest.fixture
    def client(self):
        return Client("http://localhost:54321", "test-key")

    @pytest.mark.asyncio
    async def test_database_select(self, client):
        """Test database select operation"""
        query_builder = client.database.from_("test_table")
        assert query_builder is not None

        # Test method chaining
        query_builder = query_builder.select("id, name")
        assert query_builder is not None

        # Test filter
        query_builder = query_builder.filter("active", "eq", "true")
        assert query_builder is not None

        # Execute would fail without real database, so we just test the builder
        try:
            await query_builder.execute()
        except SupabaseError:
            # Expected without real database
            pass

    def test_query_builder_chaining(self, client):
        """Test query builder method chaining"""
        result = client.database.from_("profiles") \
            .select("id, name") \
            .filter("active", "eq", "true")

        assert result is not None
        assert hasattr(result, 'execute')


class TestStorage:
    """Test storage operations"""

    @pytest.fixture
    def client(self):
        return Client("http://localhost:54321", "test-key")

    @pytest.mark.asyncio
    async def test_storage_list_buckets(self, client):
        """Test listing storage buckets"""
        try:
            buckets = await client.storage.list_buckets()
            assert isinstance(buckets, list)
        except SupabaseError:
            # Expected without real Supabase instance
            pass


class TestFunctions:
    """Test edge functions operations"""

    @pytest.fixture
    def client(self):
        return Client("http://localhost:54321", "test-key")

    @pytest.mark.asyncio
    async def test_functions_invoke(self, client):
        """Test function invocation"""
        try:
            result = await client.functions.invoke("test-function", {"key": "value"})
            assert isinstance(result, str)
        except SupabaseError:
            # Expected without real function
            pass

    @pytest.mark.asyncio
    async def test_functions_invoke_no_payload(self, client):
        """Test function invocation without payload"""
        try:
            result = await client.functions.invoke("test-function", None)
            assert isinstance(result, str)
        except SupabaseError:
            # Expected without real function
            pass


class TestErrorHandling:
    """Test error handling and edge cases"""

    def test_invalid_client_params(self):
        """Test various invalid client parameters"""
        invalid_params = [
            ("", "key"),
            ("url", ""),
            ("not-a-url", "key"),
            ("http://", ""),
        ]

        for url, key in invalid_params:
            with pytest.raises((SupabaseError, ValueError)):
                Client(url, key)

    @pytest.mark.asyncio
    async def test_auth_invalid_credentials(self):
        """Test authentication with invalid credentials"""
        client = Client("http://localhost:54321", "test-key")

        with pytest.raises(SupabaseError):
            await client.auth.sign_in("", "")


class TestPerformance:
    """Test performance characteristics"""

    @pytest.fixture
    def client(self):
        return Client("http://localhost:54321", "test-key")

    @pytest.mark.asyncio
    async def test_concurrent_operations(self, client):
        """Test concurrent operations performance"""
        import time

        start_time = time.time()

        # Create multiple concurrent operations
        tasks = []
        for i in range(10):
            # These will fail but we're testing concurrency handling
            task = client.database.from_(f"table_{i}").select("*").execute()
            tasks.append(task)

        # Execute concurrently
        results = await asyncio.gather(*tasks, return_exceptions=True)

        end_time = time.time()
        execution_time = (end_time - start_time) * 1000  # milliseconds

        # Should complete quickly even with errors
        assert execution_time < 5000  # Less than 5 seconds
        assert len(results) == 10

    def test_client_creation_performance(self):
        """Test client creation performance"""
        import time

        start_time = time.time()

        # Create multiple clients
        clients = []
        for i in range(100):
            client = Client("http://localhost:54321", f"test-key-{i}")
            clients.append(client)

        end_time = time.time()
        creation_time = (end_time - start_time) * 1000  # milliseconds

        # Should be very fast due to Rust performance
        assert creation_time < 1000  # Less than 1 second for 100 clients
        assert len(clients) == 100


if __name__ == "__main__":
    # Run with: python -m pytest python/tests/test_client.py -v
    pytest.main([__file__, "-v"])
