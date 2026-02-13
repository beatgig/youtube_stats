use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::exceptions::PyValueError;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// YouTube API Response Structures
#[derive(Debug, Deserialize, Serialize)]
struct YouTubeChannelResponse {
    #[serde(default)]
    items: Vec<YouTubeChannel>,
    #[serde(rename = "pageInfo")]
    page_info: Option<PageInfo>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct YouTubeChannel {
    id: String,
    snippet: ChannelSnippet,
    statistics: ChannelStatistics,
    #[serde(rename = "contentDetails")]
    content_details: Option<ContentDetails>,
    #[serde(rename = "brandingSettings")]
    branding_settings: Option<BrandingSettings>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ContentDetails {
    #[serde(rename = "relatedPlaylists")]
    related_playlists: RelatedPlaylists,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct RelatedPlaylists {
    uploads: Option<String>,
    likes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BrandingSettings {
    channel: Option<ChannelBranding>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChannelBranding {
    title: Option<String>,
    description: Option<String>,
    keywords: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Thumbnails {
    default: Option<Thumbnail>,
    medium: Option<Thumbnail>,
    high: Option<Thumbnail>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize)]
struct YouTubeSearchResultId {
    #[serde(rename = "kind")]
    kind: String,
    #[serde(rename = "channelId")]
    channel_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct YouTubeSearchResultSnippet {
    title: String,
    description: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
}

#[derive(Debug, Deserialize)]
struct YouTubeSearchResult {
    id: YouTubeSearchResultId,
    snippet: YouTubeSearchResultSnippet,
}

#[derive(Debug, Deserialize)]
struct YouTubeSearchResponse {
    items: Vec<YouTubeSearchResult>,
}

// ============================================================================
// NEW: URL Parsing Helper Functions
// ============================================================================

#[derive(Debug, Clone)]
enum IdentifierType {
    ChannelId,
    Handle,
    Username,
}

/// Parse YouTube URL or identifier to extract channel ID/handle
/// This fixes the 403 errors you were seeing!
fn parse_youtube_identifier(input: &str) -> (String, IdentifierType) {
    let input = input.trim();
    
    // Handle @ handles directly
    if input.starts_with('@') {
        return (input.to_string(), IdentifierType::Handle);
    }
    
    // Parse URLs
    if input.starts_with("http://") || input.starts_with("https://") {
        // https://www.youtube.com/channel/UCxxxxx
        if input.contains("/channel/") {
            if let Some(id) = input.split("/channel/").nth(1) {
                let id = id.split('?').next().unwrap_or(id).trim_end_matches('/');
                if id.starts_with("UC") {
                    return (id.to_string(), IdentifierType::ChannelId);
                }
            }
        }
        
        // https://www.youtube.com/@handle
        if input.contains("/@") {
            if let Some(handle) = input.split("/@").nth(1) {
                let handle = handle.split('?').next().unwrap_or(handle).trim_end_matches('/');
                return (format!("@{}", handle), IdentifierType::Handle);
            }
        }
        
        // https://www.youtube.com/user/username
        if input.contains("/user/") {
            if let Some(username) = input.split("/user/").nth(1) {
                let username = username.split('?').next().unwrap_or(username).trim_end_matches('/');
                return (username.to_string(), IdentifierType::Username);
            }
        }
        
        // https://youtube.com/customname
        if let Some(last_part) = input.split("youtube.com/").nth(1) {
            let last_part = last_part
                .trim_start_matches("c/")
                .split('?').next().unwrap_or("")
                .trim_end_matches('/');
            if !last_part.is_empty() && !last_part.contains('/') {
                return (last_part.to_string(), IdentifierType::Username);
            }
        }
    }
    
    // Direct channel ID
    if input.starts_with("UC") && input.len() > 20 {
        return (input.to_string(), IdentifierType::ChannelId);
    }
    
    // Default to username
    (input.to_string(), IdentifierType::Username)
}

// ============================================================================
// ORIGINAL FUNCTION: fetch_channel_by_url (IMPROVED with URL parsing)
// ============================================================================

fn fetch_channel_by_url(
    client: &Client,
    api_key: &str,
    channel_identifier: &str,
) -> PyResult<YouTubeChannel> {
    let base_url = "https://www.googleapis.com/youtube/v3";
    
    // NEW: Parse the identifier to handle different URL formats
    let (parsed, id_type) = parse_youtube_identifier(channel_identifier);
    
    let mut url = format!(
        "{}/channels?part=snippet,statistics,contentDetails,brandingSettings&key={}",
        base_url, api_key
    );

    match id_type {
        IdentifierType::ChannelId => {
            // Direct channel ID lookup
            url.push_str(&format!("&id={}", parsed));
        },
        IdentifierType::Handle => {
            // Handle (@username) - try forHandle first
            let handle = parsed.trim_start_matches('@');
            let search_url = format!(
                "{}/channels?part=snippet,statistics,contentDetails,brandingSettings&forHandle={}&key={}",
                base_url, handle, api_key
            );
            
            let resp = client.get(&search_url)
                .header("Accept", "application/json")
                .send()
                .map_err(|e| PyValueError::new_err(format!("Request failed: {}", e)))?;
            
            if resp.status().is_success() {
                let data: YouTubeChannelResponse = resp.json()
                    .map_err(|e| PyValueError::new_err(format!("Failed to parse: {}", e)))?;
                
                if let Some(channel) = data.items.into_iter().next() {
                    return Ok(channel);
                }
            }
            
            // Fall back to search if forHandle didn't work
            let search_url = format!(
                "{}/search?part=snippet&type=channel&q={}&key={}",
                base_url, handle, api_key
            );
            let search_resp = client.get(&search_url)
                .header("Accept", "application/json")
                .send()
                .map_err(|e| PyValueError::new_err(format!("Search request failed: {}", e)))?;
            
            if !search_resp.status().is_success() {
                return Err(PyValueError::new_err(format!("Search failed: {}", search_resp.status())));
            }
            
            let search_data: YouTubeSearchResponse = search_resp.json()
                .map_err(|e| PyValueError::new_err(format!("Failed to parse search results: {}", e)))?;
            
            let first_channel = search_data.items.into_iter().next()
                .ok_or_else(|| PyValueError::new_err("Channel not found via handle"))?;
            
            if let Some(channel_id) = &first_channel.id.channel_id {
                return fetch_channel_by_url(client, api_key, channel_id);
            } else {
                return Err(PyValueError::new_err("Channel ID not found in search result"));
            }
        },
        IdentifierType::Username => {
            // Old username - try forUsername
            url.push_str(&format!("&forUsername={}", parsed));
        }
    }

    let resp = client.get(&url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| PyValueError::new_err(format!("Request failed: {}", e)))?;

    if !resp.status().is_success() {
        return Err(PyValueError::new_err(format!("Failed to fetch channel: {}", resp.status())));
    }

    let data: YouTubeChannelResponse = resp.json()
        .map_err(|e| PyValueError::new_err(format!("Failed to parse channel data: {}", e)))?;

    data.items.into_iter().next()
        .ok_or_else(|| PyValueError::new_err("Channel not found"))
}

// ============================================================================
// ORIGINAL FUNCTION: get_youtube_channel_stats (KEPT INTACT)
// ============================================================================

/// Get YouTube channel statistics and recent videos
/// 
/// # Arguments
/// * `channel_identifier` - Can be channel ID, username, custom URL, or full YouTube URL
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
    
    // Fetch the channel (now with improved URL parsing!)
    let channel = fetch_channel_by_url(&client, &api_key, &channel_identifier)?;

    // Get recent videos if we have an uploads playlist
    let mut recent_videos = Vec::new();
    
    if videos_to_fetch > 0 && channel.content_details.is_some() {
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
            let video_id = video.id.video_id.clone();
            
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

// ============================================================================
// NEW FUNCTION: Batch processing for quota efficiency
// ============================================================================

/// Fetch up to 50 channels in ONE API call (1 quota unit!)
fn fetch_channels_by_ids(
    client: &Client,
    api_key: &str,
    channel_ids: &[String],
) -> PyResult<HashMap<String, YouTubeChannel>> {
    if channel_ids.is_empty() {
        return Ok(HashMap::new());
    }
    
    let ids_param = channel_ids.join(",");
    let url = format!(
        "https://www.googleapis.com/youtube/v3/channels?part=snippet,statistics,contentDetails,brandingSettings&id={}&key={}",
        ids_param, api_key
    );
    
    let resp = client.get(&url)
        .header("Accept", "application/json")
        .send()
        .map_err(|e| PyValueError::new_err(format!("Batch request failed: {}", e)))?;
    
    if !resp.status().is_success() {
        let status = resp.status();
        let error_text = resp.text().unwrap_or_else(|_| "Unknown".to_string());
        return Err(PyValueError::new_err(format!(
            "Batch fetch failed ({}): {}", status, error_text
        )));
    }
    
    let data: YouTubeChannelResponse = resp.json()
        .map_err(|e| PyValueError::new_err(format!("Parse error: {}", e)))?;
    
    let mut results = HashMap::new();
    for channel in data.items {
        results.insert(channel.id.clone(), channel);
    }
    Ok(results)
}

/// NEW: Batch fetch multiple YouTube channels efficiently
/// 
/// # Arguments
/// * `channel_identifiers` - List of YouTube URLs/IDs/handles
/// * `api_key` - YouTube Data API v3 key
/// * `artist_ids` - Optional list of artist IDs (must match length of channel_identifiers)
/// 
/// # Returns
/// List of dicts with artist_id, channel stats, and success status
/// 
/// Quota usage: 1 unit per 50 channel IDs (50x more efficient!)
#[pyfunction]
pub fn get_youtube_channels_batch(
    channel_identifiers: Vec<String>,
    api_key: String,
    artist_ids: Option<Vec<String>>,
) -> PyResult<PyObject> {
    let client = Client::new();
    
    // Validate artist_ids length if provided
    if let Some(ref ids) = artist_ids {
        if ids.len() != channel_identifiers.len() {
            return Err(PyValueError::new_err(format!(
                "artist_ids length ({}) must match channel_identifiers length ({})",
                ids.len(), channel_identifiers.len()
            )));
        }
    }
    
    // Parse all inputs and categorize
    let mut channel_ids = Vec::new();
    let mut non_id_lookups = Vec::new();
    let mut input_to_parsed: HashMap<String, (String, IdentifierType)> = HashMap::new();
    
    for input in &channel_identifiers {
        let (parsed, id_type) = parse_youtube_identifier(input);
        
        if matches!(id_type, IdentifierType::ChannelId) {
            channel_ids.push(parsed.clone());
        } else {
            non_id_lookups.push((input.clone(), parsed.clone(), id_type.clone()));
        }
        
        // Clone both values here since we've potentially moved id_type above
        input_to_parsed.insert(input.clone(), (parsed.clone(), id_type.clone()));
    }
    
    // Batch fetch all channel IDs (50 at a time)
    let mut all_channels: HashMap<String, YouTubeChannel> = HashMap::new();
    
    for chunk in channel_ids.chunks(50) {
        match fetch_channels_by_ids(&client, &api_key, chunk) {
            Ok(channels) => {
                all_channels.extend(channels);
            },
            Err(e) => {
                eprintln!("Error fetching batch: {}", e);
            }
        }
    }
    
    // For non-IDs, fetch individually using the original function
    for (original_input, _parsed, _id_type) in non_id_lookups {
        match fetch_channel_by_url(&client, &api_key, &original_input) {
            Ok(channel) => {
                all_channels.insert(channel.id.clone(), channel);
            },
            Err(e) => {
                eprintln!("Error fetching {}: {}", original_input, e);
            }
        }
    }
    
    // Build Python response as a list
    Python::with_gil(|py| {
        let py_results = PyList::empty(py);
        
        for (idx, input) in channel_identifiers.iter().enumerate() {
            let py_dict = PyDict::new(py);
            
            // Add artist_id if provided
            if let Some(ref ids) = artist_ids {
                py_dict.set_item("artist_id", &ids[idx])?;
            }
            
            py_dict.set_item("input_identifier", input)?;
            
            // Find matching channel
            let channel_opt = if let Some((parsed, id_type)) = input_to_parsed.get(input) {
                if matches!(id_type, IdentifierType::ChannelId) {
                    all_channels.get(parsed)
                } else {
                    all_channels.values().find(|c| {
                        c.snippet.custom_url.as_ref().map_or(false, |u| 
                            u.trim_start_matches('@') == parsed.trim_start_matches('@')
                        )
                    })
                }
            } else {
                None
            };
            
            if let Some(channel) = channel_opt {
                py_dict.set_item("success", true)?;
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
                
                // Subscriber count
                if !channel.statistics.hidden_subscriber_count {
                    if let Some(sub_count) = &channel.statistics.subscriber_count {
                        if let Ok(count) = sub_count.parse::<u64>() {
                            py_dict.set_item("subscriber_count", count)?;
                        }
                    }
                } else {
                    py_dict.set_item("subscriber_count", py.None())?;
                    py_dict.set_item("subscriber_count_hidden", true)?;
                }
                
                if let Some(view_count) = &channel.statistics.view_count {
                    if let Ok(views) = view_count.parse::<u64>() {
                        py_dict.set_item("total_view_count", views)?;
                    }
                }
                
                if let Some(video_count_str) = &channel.statistics.video_count {
                    if let Ok(videos) = video_count_str.parse::<u32>() {
                        py_dict.set_item("video_count", videos)?;
                    }
                }
                
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
                
                py_dict.set_item("channel_url", 
                    format!("https://www.youtube.com/channel/{}", channel.id))?;
            } else {
                py_dict.set_item("success", false)?;
                py_dict.set_item("error", "Channel not found or API error")?;
            }
            
            py_results.append(py_dict)?;
        }
        
        Ok(py_results.into())
    })
}

// ============================================================================
// ORIGINAL FUNCTION: search_youtube_channels (KEPT INTACT)
// ============================================================================

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

    let search_results: YouTubeSearchResponse = response.json()
        .map_err(|e| PyValueError::new_err(format!("Failed to parse search results: {}", e)))?;
    
    Python::with_gil(|py| {
        let py_dicts: Vec<Py<PyDict>> = search_results.items.iter()
            .filter_map(|item| {
                if let Some(channel_id) = &item.id.channel_id {
                    let channel_dict = PyDict::new(py);
                    channel_dict.set_item("channel_id", channel_id).unwrap();
                    channel_dict.set_item("title", &item.snippet.title).unwrap();
                    channel_dict.set_item("description", &item.snippet.description).unwrap();
                    channel_dict.set_item(
                        "channel_url",
                        format!("https://www.youtube.com/channel/{}", channel_id)
                    ).unwrap();
                    Some(channel_dict.into())
                } else {
                    None
                }
            })
            .collect();

        Ok(PyList::new(py, py_dicts).into())
    })
}