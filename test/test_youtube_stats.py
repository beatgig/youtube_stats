import os
import pytest
from youtube_stats import account
from youtube_stats import auth
from dotenv import load_dotenv

load_dotenv()

def test_youtube_channel_stats():
    """Test fetching YouTube channel statistics."""
    youtube_api_key_from_os = os.environ.get("youtube_api_key_from_os")
    if not youtube_api_key_from_os:
        pytest.skip("youtube_api_key_from_os environment variable not set")
    
    print(f"YouTube API Key: {youtube_api_key_from_os[:10]}..." if youtube_api_key_from_os else "Not found")
    assert youtube_api_key_from_os, "YouTube API key is required"

    youtube_api_key = auth.get_youtube_api_key()

    assert youtube_api_key, "YouTube API key is required, Found in os.environ.get but not in auth.get_youtube_api_key"
    assert youtube_api_key == youtube_api_key_from_os, "YouTube API key is required, Found in os.environ.get and auth.get_youtube_api_key, but didnt match"
    
    
    test_channels = [
        "@mkbhd",  # Modern @ handle
        "UCBJycsmduvYEL83R_U4JriQ",  # Channel ID
        "marquesbrownlee"  # Legacy username (if applicable)
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
        
        assert "thumbnails" in stats, "thumbnails is required"
        assert isinstance(stats["thumbnails"], dict), "thumbnails should be a dictionary"
        
        assert "recent_videos" in stats, "recent_videos is required"
        assert isinstance(stats["recent_videos"], list), "recent_videos should be a list"
        
        if len(stats["recent_videos"]) > 0:
            first_video = stats["recent_videos"][0]
            assert "video_id" in first_video, "video_id is required in video"
            assert "title" in first_video, "title is required in video"
            assert "published_at" in first_video, "published_at is required in video"
            assert "video_url" in first_video, "video_url is required in video"
            
            if "view_count" in first_video:
                assert isinstance(first_video["view_count"], int), "view_count should be an integer"
            if "like_count" in first_video:
                assert isinstance(first_video["like_count"], int), "like_count should be an integer"
            if "comment_count" in first_video:
                assert isinstance(first_video["comment_count"], int), "comment_count should be an integer"
        
        assert "total_recent_views" in stats, "total_recent_views is required"
        assert "total_recent_likes" in stats, "total_recent_likes is required"
        assert "total_recent_comments" in stats, "total_recent_comments is required"
        
        print(f"Channel: {stats['channel_title']}")
        print(f"Video count: {stats['video_count']}")
        print(f"Total views: {stats['total_view_count']:,}")
        print(f"Recent videos fetched: {len(stats['recent_videos'])}")
        print(f"Total recent views: {stats['total_recent_views']:,}")


def test_search_youtube_channels():
    """Test searching for YouTube channels."""
    youtube_api_key = auth.get_youtube_api_key()
    if not youtube_api_key:
        pytest.skip("YOUTUBE_API_KEY environment variable not set")

    assert youtube_api_key, "YouTube API key is required to be not None or False, Empty List, etc. should be string"
    
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
    if not youtube_api_key:
        pytest.skip("YOUTUBE_API_KEY environment variable not set")

    assert youtube_api_key, "YouTube API key is required to be not None or False, Empty List, etc. should be string"
    
    print("\nTesting error handling")
    
    with pytest.raises(Exception) as exc_info:
        stats = account.get_youtube_channel_stats(
            channel_identifier="@mkbhd",
            api_key="invalid_api_key_12345",
            video_count=5
        )
    
    error_message = str(exc_info.value)
    print(f"Invalid API key error: {error_message}")
    assert "API" in error_message or "401" in error_message or "403" in error_message, \
        "Should get API error with invalid key"
    
    with pytest.raises(Exception) as exc_info:
        stats = account.get_youtube_channel_stats(
            channel_identifier="this_channel_definitely_does_not_exist_12345",
            api_key=youtube_api_key,
            video_count=5
        )
    
    error_message = str(exc_info.value)
    print(f"Non-existent channel error: {error_message}")
    assert "not found" in error_message.lower() or "404" in error_message, \
        "Should get not found error for non-existent channel"


def test_different_video_counts():
    """Test fetching different numbers of recent videos."""
    youtube_api_key = auth.get_youtube_api_key()
    if not youtube_api_key:
        pytest.skip("YOUTUBE_API_KEY environment variable not set")

    assert youtube_api_key, "YouTube API key is required to be not None or False, Empty List, etc. should be string" 
     
    print("\nTesting different video counts")
    
    test_counts = [1, 5, 20]
    
    for count in test_counts:
        print(f"\nFetching {count} videos")
        
        stats = account.get_youtube_channel_stats(
            channel_identifier="@mkbhd",
            api_key=youtube_api_key,
            video_count=count
        )
        
        video_count = len(stats["recent_videos"])
        print(f"Requested: {count}, Received: {video_count}")
        
        assert video_count <= count, f"Should not return more than {count} videos"
        assert video_count >= 0, "Video count should be non-negative"


def test_channel_without_recent_videos():
    """Test handling channels that might not have recent videos."""
    youtube_api_key = auth.get_youtube_api_key()
    if not youtube_api_key:
        pytest.skip("YOUTUBE_API_KEY environment variable not set")

    assert youtube_api_key, "YouTube API key is required to be not None or False, Empty List, etc. should be string" 
    print("\nTesting channel data retrieval (even without videos)")
    
    stats = account.get_youtube_channel_stats(
        channel_identifier="@youtube",
        api_key=youtube_api_key,
        video_count=1
    )
    
    assert "channel_id" in stats
    assert "channel_title" in stats
    assert "recent_videos" in stats
    assert isinstance(stats["recent_videos"], list)
    
    if len(stats["recent_videos"]) == 0:
        assert stats["total_recent_views"] == 0
        assert stats["total_recent_likes"] == 0
        assert stats["total_recent_comments"] == 0
    
    print("Channel info retrieved successfully")
    print(f"Videos found: {len(stats['recent_videos'])}")

