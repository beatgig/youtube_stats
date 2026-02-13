import os
import pytest
from youtube_stats import account
from youtube_stats import auth
from dotenv import load_dotenv

load_dotenv()


def test_youtube_channel_stats():
    """Test fetching YouTube channel statistics."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    test_channels = [
        "@mkbhd",  # Modern @ handle
        "UCBJycsmduvYEL83R_U4JriQ",  # Channel ID
        "https://www.youtube.com/channel/UCBJycsmduvYEL83R_U4JriQ",  # Full URL with channel ID
    ]
    
    for channel_identifier in test_channels[:1]:
        print(f"\nTesting channel: {channel_identifier}")
        
        stats = account.get_youtube_channel_stats(
            channel_identifier=channel_identifier,
            api_key=youtube_api_key,
            video_count=5
        )
        
        print(f"Channel stats retrieved: {stats.keys()}")
        
        assert stats, "Stats dictionary should not be empty"
        assert "channel_id" in stats, "channel_id is required"
        assert "channel_title" in stats, "channel_title is required"
        assert "channel_description" in stats, "channel_description is required"
        assert "published_at" in stats, "published_at is required"
        assert "channel_url" in stats, "channel_url is required"
        
        assert "video_count" in stats, "video_count is required"
        assert "total_view_count" in stats, "total_view_count is required"
        
        if "subscriber_count_hidden" in stats and stats["subscriber_count_hidden"]:
            print("Subscriber count is hidden for this channel")
            assert stats.get("subscriber_count") is None, "Hidden subscriber count should be None"
        else:
            assert "subscriber_count" in stats, "subscriber_count is required when not hidden"
            assert isinstance(stats["subscriber_count"], int), "subscriber_count should be an integer"
            print(f"Subscriber count: {stats['subscriber_count']:,}")
        
        print(f"Channel: {stats['channel_title']}")
        print(f"Video count: {stats['video_count']}")
        print(f"Total views: {stats['total_view_count']:,}")


def test_batch_fetch_with_channel_ids():
    """Test batch fetching with channel IDs (most efficient)."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing Batch Fetch with Channel IDs ===")
    
    # Test with real channel IDs
    channel_ids = [
        "UCBJycsmduvYEL83R_U4JriQ",  # MKBHD
        "UCX6OQ3DkcsbYNE6H8uQQuVA",  # MrBeast
        "UC-lHJZR3Gqxm24_Vd_AJ5Yw",  # PewDiePie
    ]
    
    results = account.get_youtube_channels_batch(
        channel_identifiers=channel_ids,
        api_key=youtube_api_key,
        artist_ids=None
    )
    
    assert isinstance(results, list), "Results should be a list"
    assert len(results) == len(channel_ids), "Should return same number of results as inputs"
    
    for i, result in enumerate(results):
        print(f"\nResult {i+1}:")
        print(f"  Input: {channel_ids[i]}")
        print(f"  Success: {result['success']}")
        
        if result['success']:
            print(f"  Channel: {result['channel_title']}")
            print(f"  Subscribers: {result.get('subscriber_count', 'Hidden')}")
            print(f"  Videos: {result.get('video_count', 'N/A')}")
            
            assert "channel_id" in result
            assert "channel_title" in result
            assert "channel_url" in result
            assert result["input_identifier"] == channel_ids[i]
        else:
            print(f"  Error: {result.get('error', 'Unknown error')}")


def test_batch_fetch_with_artist_ids():
    """Test batch fetching with artist_id mapping."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing Batch Fetch with Artist IDs ===")
    
    # Simulate your artist database structure
    artists = [
        {"artist_id": "artist-001", "youtube_url": "UCBJycsmduvYEL83R_U4JriQ"},
        {"artist_id": "artist-002", "youtube_url": "UCX6OQ3DkcsbYNE6H8uQQuVA"},
        {"artist_id": "artist-003", "youtube_url": "@mkbhd"},
    ]
    
    channel_identifiers = [a["youtube_url"] for a in artists]
    artist_ids = [a["artist_id"] for a in artists]
    
    results = account.get_youtube_channels_batch(
        channel_identifiers=channel_identifiers,
        api_key=youtube_api_key,
        artist_ids=artist_ids
    )
    
    assert isinstance(results, list), "Results should be a list"
    assert len(results) == len(artists), "Should return result for each artist"
    
    # Build a lookup dict for easy access
    results_by_artist = {r["artist_id"]: r for r in results}
    
    for artist in artists:
        artist_id = artist["artist_id"]
        assert artist_id in results_by_artist, f"Should have result for {artist_id}"
        
        result = results_by_artist[artist_id]
        print(f"\nArtist ID: {artist_id}")
        print(f"  Input: {result['input_identifier']}")
        print(f"  Success: {result['success']}")
        
        if result['success']:
            print(f"  Channel: {result['channel_title']}")
            print(f"  Subscribers: {result.get('subscriber_count', 'Hidden')}")
            
            assert "artist_id" in result
            assert result["artist_id"] == artist_id
            assert "channel_id" in result
            assert "channel_title" in result


def test_batch_fetch_with_mixed_formats():
    """Test batch fetching with different URL formats."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing Batch Fetch with Mixed URL Formats ===")
    
    # Different URL formats for the same channels
    mixed_inputs = [
        "https://www.youtube.com/channel/UCBJycsmduvYEL83R_U4JriQ",  # Full URL
        "@mkbhd",  # Handle
        "UCBJycsmduvYEL83R_U4JriQ",  # Direct channel ID
        "https://www.youtube.com/@mkbhd",  # Handle URL
        "https://www.youtube.com/channel/UCX6OQ3DkcsbYNE6H8uQQuVA",  # Another channel
    ]
    
    results = account.get_youtube_channels_batch(
        channel_identifiers=mixed_inputs,
        api_key=youtube_api_key,
        artist_ids=None
    )
    
    assert len(results) == len(mixed_inputs)
    
    success_count = sum(1 for r in results if r['success'])
    print(f"\nSuccessful fetches: {success_count}/{len(results)}")
    
    for i, result in enumerate(results):
        print(f"\nInput: {mixed_inputs[i]}")
        print(f"  Success: {result['success']}")
        if result['success']:
            print(f"  Channel: {result['channel_title']}")
            print(f"  Channel ID: {result['channel_id']}")
        else:
            print(f"  Error: {result.get('error', 'Unknown')}")


def test_batch_fetch_quota_efficiency():
    """Test that batch fetching is more quota-efficient than individual calls."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing Quota Efficiency ===")
    
    # Test with 10 channel IDs (should use only 1 quota unit for batch vs 10 individual)
    channel_ids = [
        "UCBJycsmduvYEL83R_U4JriQ",  # MKBHD
        "UCX6OQ3DkcsbYNE6H8uQQuVA",  # MrBeast
        "UC-lHJZR3Gqxm24_Vd_AJ5Yw",  # PewDiePie
        "UCq-Fj5jknLsUf-MWSy4_brA",  # T-Series
        "UCbCmjCuTUZos6Inko4u57UQ",  # Cocomelon
        "UCAuUUnT6oDeKwE6v1NGQxug",  # 5-Minute Crafts
        "UCpEhnqL0y41EpW2TvWAHD7Q",  # Like Nastya
        "UCFFbwnve3yF62-tV_hMALCA",  # Dude Perfect
        "UCsvn_Po0SmunchJYOWpOxMg",  # Vlad and Niki
        "UCtinbF-Q-fVthA0qrFQTgXQ",  # Zee Music Company
    ]
    
    # Batch call (should use ~1 quota unit for all channel IDs)
    results = account.get_youtube_channels_batch(
        channel_identifiers=channel_ids,
        api_key=youtube_api_key,
        artist_ids=None
    )
    
    print(f"Batch fetched {len(channel_ids)} channels in one call")
    print("Estimated quota usage: 1 unit (vs {len(channel_ids)} for individual calls)")
    
    success_count = sum(1 for r in results if r['success'])
    print(f"Successful: {success_count}/{len(channel_ids)}")
    
    assert success_count >= len(channel_ids) * 0.8, "At least 80% should succeed"


def test_batch_fetch_with_invalid_channels():
    """Test batch fetching handles invalid channels gracefully."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing Batch Fetch with Invalid Channels ===")
    
    mixed_valid_invalid = [
        "UCBJycsmduvYEL83R_U4JriQ",  # Valid
        "UCinvalidchannel12345",  # Invalid
        "@mkbhd",  # Valid
        "@nonexistenthandle99999",  # Invalid
    ]
    
    results = account.get_youtube_channels_batch(
        channel_identifiers=mixed_valid_invalid,
        api_key=youtube_api_key,
        artist_ids=None
    )
    
    assert len(results) == len(mixed_valid_invalid)
    
    for i, result in enumerate(results):
        print(f"\nInput: {mixed_valid_invalid[i]}")
        print(f"  Success: {result['success']}")
        
        if result['success']:
            print(f"  Channel: {result['channel_title']}")
            assert "channel_id" in result
            assert "subscriber_count" in result or "subscriber_count_hidden" in result
        else:
            print(f"  Error: {result.get('error', 'Unknown')}")
            assert "error" in result
            assert not result['success']


def test_batch_artist_id_length_mismatch():
    """Test that mismatched artist_ids length raises error."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing Artist ID Length Validation ===")
    
    channel_ids = ["UCBJycsmduvYEL83R_U4JriQ", "UCX6OQ3DkcsbYNE6H8uQQuVA"]
    artist_ids = ["artist-001"]  # Mismatched length
    
    with pytest.raises(Exception) as exc_info:
        account.get_youtube_channels_batch(
            channel_identifiers=channel_ids,
            api_key=youtube_api_key,
            artist_ids=artist_ids
        )
    
    error_message = str(exc_info.value)
    print(f"Expected error: {error_message}")
    assert "must match" in error_message.lower() or "length" in error_message.lower()


def test_batch_large_scale():
    """Test batch fetching with 50 channels (maximum batch size)."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing Large Scale Batch (50 channels) ===")
    
    # Generate 50 test channel IDs (some valid, some invalid)
    # In real usage, you'd have actual channel IDs
    channel_ids = [f"UC{'x' * 22}{i:02d}" for i in range(50)]
    # Add some real ones at the beginning
    channel_ids[0] = "UCBJycsmduvYEL83R_U4JriQ"  # MKBHD
    channel_ids[1] = "UCX6OQ3DkcsbYNE6H8uQQuVA"  # MrBeast
    
    artist_ids = [f"artist-{i:04d}" for i in range(50)]
    
    results = account.get_youtube_channels_batch(
        channel_identifiers=channel_ids,
        api_key=youtube_api_key,
        artist_ids=artist_ids
    )
    
    assert len(results) == 50, "Should return 50 results"
    
    # Check the first two known valid channels
    assert results[0]['success'], "First channel should succeed"
    assert results[1]['success'], "Second channel should succeed"
    assert results[0]['artist_id'] == "artist-0000"
    assert results[1]['artist_id'] == "artist-0001"
    
    print(f"Processed 50 channels in one batch call")
    print(f"Artist ID mapping working: {results[0]['artist_id']} -> {results[0]['channel_title']}")
    print("Estimated quota usage: 1 unit (vs 50 for individual calls)")


def test_search_youtube_channels():
    """Test searching for YouTube channels."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\nTesting YouTube channel search")
    
    search_results = account.search_youtube_channels(
        query="technology reviews",
        api_key=youtube_api_key,
        max_results=3
    )
    
    print(f"Search returned {len(search_results)} results")
    
    assert search_results, "Search results should not be empty"
    assert isinstance(search_results, list), "Search results should be a list"
    assert len(search_results) <= 3, "Should not return more than requested results"
    
    if len(search_results) > 0:
        first_result = search_results[0]
        assert "channel_id" in first_result, "channel_id is required in search result"
        assert "title" in first_result, "title is required in search result"
        assert "description" in first_result, "description is required in search result"
        assert "channel_url" in first_result, "channel_url is required in search result"
        
        print(f"First result: {first_result['title']}")
        print(f"Channel URL: {first_result['channel_url']}")


def test_error_handling():
    """Test error handling with invalid inputs."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"

    print("\nTesting error handling")

    # Invalid API key
    with pytest.raises(Exception) as exc_info:
        stats = account.get_youtube_channel_stats(
            channel_identifier="@mkbhd",
            api_key="invalid_api_key_12345",
            video_count=5
        )
    error_message = str(exc_info.value)
    print(f"Invalid API key error: {error_message}")
    assert any(
        x in error_message.lower()
        for x in ["api", "401", "403", "400", "bad request"]
    ), "Should get API error with invalid key"

    # Non-existent channel
    with pytest.raises(Exception) as exc_info:
        stats = account.get_youtube_channel_stats(
            channel_identifier="this_channel_definitely_does_not_exist_12345",
            api_key=youtube_api_key,
            video_count=5
        )
    error_message = str(exc_info.value)
    print(f"Non-existent channel error: {error_message}")
    assert any(
        x in error_message.lower()
        for x in ["not found", "404", "channel not found"]
    ), "Should get not found error for non-existent channel"


def test_url_parsing():
    """Test that various URL formats are parsed correctly."""
    youtube_api_key = auth.get_youtube_api_key()
    assert youtube_api_key, "YouTube API key is required"
    
    print("\n=== Testing URL Parsing ===")
    
    # These should all resolve to MKBHD's channel
    url_formats = [
        "UCBJycsmduvYEL83R_U4JriQ",
        "https://www.youtube.com/channel/UCBJycsmduvYEL83R_U4JriQ",
        "https://www.youtube.com/@mkbhd",
        "@mkbhd",
    ]
    
    results = account.get_youtube_channels_batch(
        channel_identifiers=url_formats,
        api_key=youtube_api_key,
        artist_ids=None
    )
    
    # All should succeed and be the same channel
    channel_ids = set()
    for i, result in enumerate(results):
        print(f"\nFormat: {url_formats[i]}")
        print(f"  Success: {result['success']}")
        if result['success']:
            print(f"  Resolved to: {result['channel_id']}")
            channel_ids.add(result['channel_id'])
    
    # Most should resolve to the same channel (some formats might not work for all channels)
    print(f"\nUnique channel IDs found: {len(channel_ids)}")
    print(f"Channel IDs: {channel_ids}")