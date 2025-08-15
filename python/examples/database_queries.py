#!/usr/bin/env python3
"""
Advanced database queries example for Supabase Python client

This example demonstrates:
- Complex queries with filters
- Joins and relationships
- Ordering and pagination
- Batch operations
- Transaction support (when implemented)

To run this example:
    python examples/database_queries.py
"""

import asyncio
import os
from datetime import datetime, timedelta
from supabase_lib_rs import Client, SupabaseError


async def main():
    print("=== Advanced Database Operations Example ===\n")

    # Initialize client
    client = Client(
        os.getenv("SUPABASE_URL", "http://localhost:54321"),
        os.getenv("SUPABASE_KEY", "your-anon-key")
    )

    print("🗄️ Demonstrating advanced database operations...\n")

    try:
        # 1. Basic CRUD operations
        print("1️⃣ Basic CRUD Operations:")

        # Create
        new_post = await client.database.from_("posts") \
            .insert({
                "title": "Python-Rust Integration",
                "content": "Leveraging Rust performance in Python applications",
                "author_id": 1,
                "published": True
            }) \
            .execute()
        print("✅ Post created")

        # Read with filters
        recent_posts = await client.database.from_("posts") \
            .select("id, title, created_at, published") \
            .filter("published", "eq", True) \
            .filter("created_at", "gte", (datetime.now() - timedelta(days=7)).isoformat()) \
            .order("created_at", desc=True) \
            .limit(10) \
            .execute()
        print(f"✅ Found {len(recent_posts)} recent published posts")

        # Update
        if recent_posts:
            post_id = recent_posts[0]["id"]
            await client.database.from_("posts") \
                .update({"view_count": 100}) \
                .filter("id", "eq", post_id) \
                .execute()
            print("✅ Post view count updated")

    except SupabaseError as e:
        print(f"❌ CRUD operation failed: {e}")

    try:
        # 2. Advanced filtering
        print("\n2️⃣ Advanced Filtering:")

        # Multiple filters
        filtered_posts = await client.database.from_("posts") \
            .select("*") \
            .filter("published", "eq", True) \
            .filter("view_count", "gte", 50) \
            .filter("title", "ilike", "%python%") \
            .execute()
        print(f"✅ Advanced filter returned {len(filtered_posts)} posts")

        # IN filter
        category_posts = await client.database.from_("posts") \
            .select("id, title, category") \
            .filter("category", "in", ["tech", "programming", "rust"]) \
            .execute()
        print(f"✅ Category filter returned {len(category_posts)} posts")

        # Range filters
        view_range = await client.database.from_("posts") \
            .select("id, title, view_count") \
            .filter("view_count", "gte", 10) \
            .filter("view_count", "lte", 1000) \
            .execute()
        print(f"✅ View range filter returned {len(view_range)} posts")

    except SupabaseError as e:
        print(f"❌ Advanced filtering failed: {e}")

    try:
        # 3. Joins and relationships
        print("\n3️⃣ Joins and Relationships:")

        # Join with profiles
        posts_with_authors = await client.database.from_("posts") \
            .select("title, content, profiles(name, avatar_url)") \
            .filter("published", "eq", True) \
            .limit(5) \
            .execute()
        print(f"✅ Joined query returned {len(posts_with_authors)} posts with author info")

        # Many-to-many relationship
        posts_with_tags = await client.database.from_("posts") \
            .select("title, post_tags(tags(name))") \
            .limit(5) \
            .execute()
        print(f"✅ Many-to-many query returned {len(posts_with_tags)} posts with tags")

    except SupabaseError as e:
        print(f"❌ Join operation failed: {e}")

    try:
        # 4. Aggregation and statistics
        print("\n4️⃣ Aggregation Operations:")

        # Count operations
        total_posts = await client.database.from_("posts") \
            .select("count") \
            .execute()
        print(f"✅ Total posts count: {total_posts[0]['count'] if total_posts else 0}")

        # Group by operations
        stats_by_author = await client.database.from_("posts") \
            .select("author_id, count(*), avg(view_count)") \
            .group_by("author_id") \
            .execute()
        print(f"✅ Author statistics: {len(stats_by_author)} authors")

    except SupabaseError as e:
        print(f"❌ Aggregation failed: {e}")

    try:
        # 5. Full-text search
        print("\n5️⃣ Full-Text Search:")

        search_results = await client.database.from_("posts") \
            .select("id, title, content") \
            .text_search("content", "rust programming language") \
            .execute()
        print(f"✅ Full-text search returned {len(search_results)} results")

        # Combined search with filters
        advanced_search = await client.database.from_("posts") \
            .select("*") \
            .text_search("title", "python rust") \
            .filter("published", "eq", True) \
            .order("created_at", desc=True) \
            .execute()
        print(f"✅ Advanced search returned {len(advanced_search)} results")

    except SupabaseError as e:
        print(f"❌ Search operation failed: {e}")

    try:
        # 6. Batch operations
        print("\n6️⃣ Batch Operations:")

        # Batch insert
        batch_data = [
            {"title": f"Batch Post {i}", "content": f"Content {i}", "author_id": 1}
            for i in range(1, 6)
        ]

        batch_result = await client.database.from_("posts") \
            .insert(batch_data) \
            .execute()
        print(f"✅ Batch insert created {len(batch_result)} records")

        # Batch update
        await client.database.from_("posts") \
            .update({"category": "batch"}) \
            .filter("title", "like", "Batch Post%") \
            .execute()
        print("✅ Batch update completed")

    except SupabaseError as e:
        print(f"❌ Batch operation failed: {e}")

    # 7. Performance metrics
    print("\n7️⃣ Performance Comparison:")

    start_time = asyncio.get_event_loop().time()

    # Concurrent queries
    concurrent_tasks = [
        client.database.from_("posts").select("id").limit(100).execute(),
        client.database.from_("profiles").select("id").limit(50).execute(),
        client.database.from_("posts").select("count").execute(),
    ]

    try:
        results = await asyncio.gather(*concurrent_tasks)
        end_time = asyncio.get_event_loop().time()

        total_time = (end_time - start_time) * 1000
        print(f"✅ 3 concurrent queries completed in {total_time:.1f}ms")
        print("   🚀 Rust-powered performance advantage!")

    except Exception as e:
        print(f"❌ Performance test failed: {e}")

    print(f"\n🎉 Advanced database operations completed!")
    print("📊 Operations demonstrated:")
    print("   • Complex filtering and querying")
    print("   • Joins and relationship queries")
    print("   • Aggregation and statistics")
    print("   • Full-text search capabilities")
    print("   • Batch operations for efficiency")
    print("   • Concurrent query execution")


if __name__ == "__main__":
    asyncio.run(main())
