use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::exceptions::PyValueError;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

// YouTube API Response Structures
#[derive(Debug, Deserialize, Serialize)]
struct YouTubeChannelResponse {
    items: Vec<YouTubeChannel>,
    #[serde(rename = "pageInfo")]
    page_info: Option<PageInfo>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct YouTubeChannel {
    id: String,
    snippet: ChannelSnippet,
    statistics: ChannelStatistics,
    #[serde(rename = "contentDetails")]
    content_details: Option<ContentDetails>,
    #[serde(rename = "brandingSettings")]
    branding_settings: Option<BrandingSettings>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChannelSnippet {
    title: String,
    description: String,
    #[serde(rename = "customUrl")]
    custom_url: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: String,
    thumbnails: Thumbnails,
    country: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChannelStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "subscriberCount")]
    subscriber_count: Option<String>,
    #[serde(rename = "hiddenSubscriberCount")]
    hidden_subscriber_count: bool,
    #[serde(rename = "videoCount")]
    video_count: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ContentDetails {
    #[serde(rename = "relatedPlaylists")]
    related_playlists: RelatedPlaylists,
}

#[derive(Debug, Deserialize, Serialize)]
struct RelatedPlaylists {
    uploads: Option<String>,
    likes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BrandingSettings {
    channel: Option<ChannelBranding>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChannelBranding {
    title: Option<String>,
    description: Option<String>,
    keywords: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Thumbnails {
    default: Option<Thumbnail>,
    medium: Option<Thumbnail>,
    high: Option<Thumbnail>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Thumbnail {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PageInfo {
    #[serde(rename = "totalResults")]
    total_results: u32,
    #[serde(rename = "resultsPerPage")]
    results_per_page: u32,
}

// Video list response structures
#[derive(Debug, Deserialize, Serialize)]
struct YouTubeVideoListResponse {
    items: Vec<YouTubeVideo>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct YouTubeVideo {
    id: VideoId,
    snippet: VideoSnippet,
    statistics: Option<VideoStatistics>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VideoId {
    #[serde(rename = "videoId")]
    video_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct VideoSnippet {
    title: String,
    description: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: String,
    thumbnails: Thumbnails,
}

#[derive(Debug, Deserialize, Serialize)]
struct VideoStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "likeCount")]
    like_count: Option<String>,
    #[serde(rename = "commentCount")]
    comment_count: Option<String>,
}

// Error response structure
#[derive(Debug, Deserialize, Serialize)]
struct YouTubeErrorResponse {
    error: YouTubeError,
}

#[derive(Debug, Deserialize, Serialize)]
struct YouTubeError {
    code: u32,
    message: String,
    errors: Vec<ErrorDetail>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ErrorDetail {
    message: String,
    domain: String,
    reason: String,
}

/// Get YouTube channel statistics and recent videos
/// 
/// # Arguments
/// * `channel_identifier` - Can be channel ID, username, or custom URL
/// * `api_key` - YouTube Data API v3 key
/// * `video_count` - Number of recent videos to fetch (default: 10)
/// 
/// # Returns
/// * PyResult<PyObject> - Dictionary containing channel stats and recent videos
#[pyfunction]
pub fn get_youtube_channel_stats(
    channel_identifier: String,
    api_key: String,
    video_count: Option<u32>,
) -> PyResult<PyObject> {
    let client = Client::new();
    let base_url = "https://www.googleapis.com/youtube/v3";
    let videos_to_fetch = video_count.unwrap_or(10);
    
    // First, try to get channel info
    // Try different approaches: by ID, by username, or by custom URL
    let mut channel_url = format!(
        "{}/channels?part=snippet,statistics,contentDetails,brandingSettings&key={}",
        base_url, api_key
    );
    
    // Check if it looks like a channel ID (starts with UC)
    if channel_identifier.starts_with("UC") {
        channel_url.push_str(&format!("&id={}", channel_identifier));
    } else if channel_identifier.starts_with("@") {
        // Handle @ usernames (custom URLs)
        let username = &channel_identifier[1..];
        channel_url = format!(
            "{}/search?part=snippet&type=channel&q={}&key={}",
            base_url, username, api_key
        );
    } else {
        // Try as username first
        channel_url.push_str(&format!("&forUsername={}", channel_identifier));
    }
    
    let response = client.get(&channel_url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| PyValueError::new_err(format!("Request failed: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text()
            .unwrap_or_else(|_| "Could not read error response".to_string());
        
        // Try to parse as error response
        if let Ok(error_resp) = serde_json::from_str::<YouTubeErrorResponse>(&error_text) {
            return Err(PyValueError::new_err(format!(
                "YouTube API Error {}: {} - {}",
                error_resp.error.code,
                error_resp.error.message,
                error_resp.error.errors.first()
                    .map(|e| e.reason.as_str())
                    .unwrap_or("Unknown reason")
            )));
        }
        
        return Err(PyValueError::new_err(format!(
            "Failed to fetch channel data: {} - {}",
            status, error_text
        )));
    }
    
    let channel_data: YouTubeChannelResponse = response.json()
        .map_err(|e| PyValueError::new_err(format!("Failed to parse channel data: {}", e)))?;
    
    // Handle search results differently if we searched by custom URL
    let channel = if channel_identifier.starts_with("@") && !channel_data.items.is_empty() {
        // For search results, we need to fetch the full channel data
        let channel_id = &channel_data.items[0].id;
        let full_channel_url = format!(
            "{}/channels?part=snippet,statistics,contentDetails,brandingSettings&id={}&key={}",
            base_url, channel_id, api_key
        );
        
        let full_response = client.get(&full_channel_url)
            .header("Accept", "application/json")
            .send()
            .map_err(|e| PyValueError::new_err(format!("Request failed: {}", e)))?;
        
        let full_channel_data: YouTubeChannelResponse = full_response.json()
            .map_err(|e| PyValueError::new_err(format!("Failed to parse channel data: {}", e)))?;
        
        full_channel_data.items.into_iter().next()
            .ok_or_else(|| PyValueError::new_err("Channel not found"))?
    } else {
        channel_data.items.into_iter().next()
            .ok_or_else(|| PyValueError::new_err("Channel not found"))?
    };
    
    // Get recent videos if we have an uploads playlist
    let mut recent_videos = Vec::new();
    
    if let Some(content_details) = &channel.content_details {
        if let Some(uploads_playlist) = &content_details.related_playlists.uploads {
            println!("Found uploads playlist");
            println!("uploads_playlist: {:?}", uploads_playlist);
            let videos_url = format!(
                "{}/search?part=id,snippet&channelId={}&maxResults={}&order=date&type=video&key={}",
                base_url, channel.id, videos_to_fetch, api_key
            );
            
            if let Ok(videos_response) = client.get(&videos_url)
                .header("Accept", "application/json")
                .send() 
            {
                if videos_response.status().is_success() {
                    if let Ok(videos_data) = videos_response.json::<YouTubeVideoListResponse>() {
                        // Get video IDs
                        let video_ids: Vec<String> = videos_data.items.iter()
                            .map(|v| v.id.video_id.clone())
                            .collect();
                        
                        if !video_ids.is_empty() {
                            // Fetch detailed statistics for these videos
                            let video_stats_url = format!(
                                "{}/videos?part=statistics,snippet&id={}&key={}",
                                base_url, video_ids.join(","), api_key
                            );
                            
                            if let Ok(stats_response) = client.get(&video_stats_url)
                                .header("Accept", "application/json")
                                .send()
                            {
                                if stats_response.status().is_success() {
                                    if let Ok(stats_data) = stats_response.json::<YouTubeVideoListResponse>() {
                                        recent_videos = stats_data.items;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Convert to Python dictionary
    Python::with_gil(|py| {
        let py_dict = PyDict::new(py);
        
        // Channel basic info
        py_dict.set_item("channel_id", &channel.id)?;
        py_dict.set_item("channel_title", &channel.snippet.title)?;
        py_dict.set_item("channel_description", &channel.snippet.description)?;
        py_dict.set_item("published_at", &channel.snippet.published_at)?;
        
        if let Some(custom_url) = &channel.snippet.custom_url {
            py_dict.set_item("custom_url", custom_url)?;
        }
        
        if let Some(country) = &channel.snippet.country {
            py_dict.set_item("country", country)?;
        }
        
        // Channel statistics
        let stats = &channel.statistics;
        
        // Parse subscriber count
        if !stats.hidden_subscriber_count {
            if let Some(sub_count) = &stats.subscriber_count {
                let subscriber_count = sub_count.parse::<u64>().unwrap_or(0);
                py_dict.set_item("subscriber_count", subscriber_count)?;
            }
        } else {
            py_dict.set_item("subscriber_count", py.None())?;
            py_dict.set_item("subscriber_count_hidden", true)?;
        }
        
        // Parse other statistics
        if let Some(view_count) = &stats.view_count {
            let views = view_count.parse::<u64>().unwrap_or(0);
            py_dict.set_item("total_view_count", views)?;
        }
        
        if let Some(video_count) = &stats.video_count {
            let videos = video_count.parse::<u32>().unwrap_or(0);
            py_dict.set_item("video_count", videos)?;
        }
        
        // Thumbnails
        let thumbnails = PyDict::new(py);
        if let Some(default) = &channel.snippet.thumbnails.default {
            thumbnails.set_item("default", &default.url)?;
        }
        if let Some(medium) = &channel.snippet.thumbnails.medium {
            thumbnails.set_item("medium", &medium.url)?;
        }
        if let Some(high) = &channel.snippet.thumbnails.high {
            thumbnails.set_item("high", &high.url)?;
        }
        py_dict.set_item("thumbnails", thumbnails)?;
        
        // Branding settings
        if let Some(branding) = &channel.branding_settings {
            if let Some(channel_branding) = &branding.channel {
                if let Some(keywords) = &channel_branding.keywords {
                    py_dict.set_item("channel_keywords", keywords)?;
                }
            }
        }
        
        // Recent videos
        let py_videos = PyList::new(py, recent_videos.iter().map(|video| {
            let video_dict = PyDict::new(py);
            
            // Try to use actual ID if available, otherwise use the nested structure
            let video_id = if video.id.video_id.is_empty() {
                // Sometimes the ID might be directly in a different field
                video.id.video_id.clone()
            } else {
                video.id.video_id.clone()
            };
            
            video_dict.set_item("video_id", &video_id).unwrap();
            video_dict.set_item("title", &video.snippet.title).unwrap();
            video_dict.set_item("published_at", &video.snippet.published_at).unwrap();
            
            if let Some(desc) = &video.snippet.description {
                video_dict.set_item("description", desc).unwrap();
            }
            
            // Video statistics
            if let Some(stats) = &video.statistics {
                if let Some(views) = &stats.view_count {
                    let view_count = views.parse::<u64>().unwrap_or(0);
                    video_dict.set_item("view_count", view_count).unwrap();
                }
                
                if let Some(likes) = &stats.like_count {
                    let like_count = likes.parse::<u64>().unwrap_or(0);
                    video_dict.set_item("like_count", like_count).unwrap();
                }
                
                if let Some(comments) = &stats.comment_count {
                    let comment_count = comments.parse::<u64>().unwrap_or(0);
                    video_dict.set_item("comment_count", comment_count).unwrap();
                }
            }
            
            // Video URL
            video_dict.set_item("video_url", format!("https://www.youtube.com/watch?v={}", video_id)).unwrap();
            
            video_dict
        }));
        
        py_dict.set_item("recent_videos", py_videos)?;
        
        // Calculate totals from recent videos
        let total_recent_views: u64 = recent_videos.iter()
            .filter_map(|v| v.statistics.as_ref())
            .filter_map(|s| s.view_count.as_ref())
            .filter_map(|v| v.parse::<u64>().ok())
            .sum();
        
        let total_recent_likes: u64 = recent_videos.iter()
            .filter_map(|v| v.statistics.as_ref())
            .filter_map(|s| s.like_count.as_ref())
            .filter_map(|l| l.parse::<u64>().ok())
            .sum();
        
        let total_recent_comments: u64 = recent_videos.iter()
            .filter_map(|v| v.statistics.as_ref())
            .filter_map(|s| s.comment_count.as_ref())
            .filter_map(|c| c.parse::<u64>().ok())
            .sum();
        
        py_dict.set_item("total_recent_views", total_recent_views)?;
        py_dict.set_item("total_recent_likes", total_recent_likes)?;
        py_dict.set_item("total_recent_comments", total_recent_comments)?;
        
        // Channel URL
        py_dict.set_item("channel_url", format!("https://www.youtube.com/channel/{}", channel.id))?;
        
        Ok(py_dict.into())
    })
}

/// Search for YouTube channels by query
/// 
/// # Arguments
/// * `query` - Search query string
/// * `api_key` - YouTube Data API v3 key  
/// * `max_results` - Maximum number of results to return (default: 5, max: 50)
///
/// # Returns
/// * PyResult<PyObject> - List of channels matching the search
#[pyfunction]
pub fn search_youtube_channels(
    query: String,
    api_key: String,
    max_results: Option<u32>,
) -> PyResult<PyObject> {
    let client = Client::new();
    let base_url = "https://www.googleapis.com/youtube/v3";
    let results_count = max_results.unwrap_or(5).min(50);
    
    let search_url = format!(
        "{}/search?part=snippet&type=channel&q={}&maxResults={}&key={}",
        base_url, query, results_count, api_key
    );
    
    let response = client.get(&search_url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| PyValueError::new_err(format!("Request failed: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text()
            .unwrap_or_else(|_| "Could not read error response".to_string());
        return Err(PyValueError::new_err(format!("Search failed: {} - {}", status, error_text)));
    }
    
    let search_results: YouTubeChannelResponse = response.json()
        .map_err(|e| PyValueError::new_err(format!("Failed to parse search results: {}", e)))?;
    
    Python::with_gil(|py| {
        let py_list = PyList::new(py, search_results.items.iter().map(|channel| {
            let channel_dict = PyDict::new(py);
            channel_dict.set_item("channel_id", &channel.id).unwrap();
            channel_dict.set_item("title", &channel.snippet.title).unwrap();
            channel_dict.set_item("description", &channel.snippet.description).unwrap();
            channel_dict.set_item("channel_url", format!("https://www.youtube.com/channel/{}", channel.id)).unwrap();
            
            if let Some(custom_url) = &channel.snippet.custom_url {
                channel_dict.set_item("custom_url", custom_url).unwrap();
            }
            
            channel_dict
        }));
        
        Ok(py_list.into())
    })
}