use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Deserialize)]
struct VideoListResponse {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    items: Vec<Video>,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    #[serde(rename = "totalResults")]
    total_results: i32,
}

#[derive(Debug, Deserialize)]
struct Video {
    id: String,
    status: Option<VideoStatus>,
    #[serde(rename = "contentDetails")]
    content_details: Option<ContentDetails>,
}

async fn check_video_exists(video_id: &str, api_key: &str) -> Result<bool, reqwest::Error> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?id={}&key={}&part=status",
        video_id, api_key
    );
    
    let response: VideoListResponse = reqwest::get(&url)
        .await?
        .json()
        .await?;
    
    Ok(!response.items.is_empty())
}

async fn check_videos_exist(video_ids: &[String], api_key: &str) -> Result<Vec<bool>, reqwest::Error> {
    let mut results = Vec::new();
    for chunk in video_ids.chunks(50) {
        let ids_param = chunk.join(",");

        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?id={}&key={}&part=status",
                video_id, api_key
        );
        
        let response: VideoListResponse = reqwest::get(&url)
            .await?
            .json()
            .await?;
        
        // Create a set of returned video IDs
        //let returned_ids: HashSet<String> = response.items
            //.iter()
            //.map(|v| v.id.clone())
            //.collect();
        
        //// Check each requested ID
        //for video_id in chunk {
            //if !returned_ids.contains(video_id) {
                //// Video not found
                //results.push(); // TODO: Return false?
            //} else {
                //// Process available video (extract from response.items)
                //let video = response.items
                    //.iter()
                    //.find(|v| v.id == *video_id)
                    //.unwrap();
                
                //// Use similar logic as check_video_availability
                //results.push((video_id.clone(), process_video_availability(video)));
            //}
        //}
    }
    
    Ok(results)
}

    

// Video Status

#[derive(Debug, Deserialize)]
struct VideoStatus {
    #[serde(rename = "privacyStatus")]
    privacy_status: String,  // "public", "private", "unlisted"
    
    #[serde(rename = "uploadStatus")]
    upload_status: String,   // "processed", "deleted", "failed", "rejected", "uploaded"
    
    embeddable: bool,
    
    #[serde(rename = "publicStatsViewable")]
    public_stats_viewable: Option<bool>,
}

fn is_video_private(video: &Video) -> bool {
    video.status
        .as_ref()
        .map(|s| s.privacy_status == "private")
        .unwrap_or(false)
}

// Video Unlisted

fn is_video_unlisted(video: &Video) -> bool {
    video.status
        .as_ref()
        .map(|s| s.privacy_status == "unlisted")
        .unwrap_or(false)
}

// can embed video

fn can_embed_video(video: &Video) -> bool {
    video.status
        .as_ref()
        .map(|s| s.embeddable)
        .unwrap_or(false)
}

// Region Restrictions

#[derive(Debug, Deserialize)]
struct ContentDetails {
    duration: String,
    
    #[serde(rename = "regionRestriction")]
    region_restriction: Option<RegionRestriction>,
    
    #[serde(rename = "contentRating")]
    content_rating: Option<ContentRating>,
}

#[derive(Debug, Deserialize)]
struct RegionRestriction {
    // If present, video is ONLY viewable in these countries
    allowed: Option<Vec<String>>,
    
    // If present, video is blocked in these countries
    blocked: Option<Vec<String>>,
}

fn is_video_available_in_region(video: &Video, user_country_code: &str) -> bool {
    if let Some(details) = &video.content_details {
        if let Some(restriction) = &details.region_restriction {
            // Check allowed list
            if let Some(allowed) = &restriction.allowed {
                return allowed.contains(&user_country_code.to_string());
            }
            
            // Check blocked list
            if let Some(blocked) = &restriction.blocked {
                return !blocked.contains(&user_country_code.to_string());
            }
        }
    }
    
    // No restrictions found
    true
}

// Content Rating

#[derive(Debug, Deserialize)]
struct ContentRating {
    #[serde(rename = "ytRating")]
    yt_rating: Option<String>,  // "ytAgeRestricted" or absent
}

fn is_age_restricted(video: &Video) -> bool {
    video.content_details
        .as_ref()
        .and_then(|cd| cd.content_rating.as_ref())
        .and_then(|cr| cr.yt_rating.as_ref())
        .map(|rating| rating == "ytAgeRestricted")
        .unwrap_or(false)
}

// Complete Check

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAvailability {
    pub exists: bool,
    pub is_embeddable: bool,
    pub is_public: bool,
    pub is_region_blocked: bool,
    pub is_age_restricted: bool,
    pub upload_status: String,
    pub unavailability_reasons: Vec<String>,
}

pub async fn check_video_availability(
    video_id: &str,
    api_key: &str,
    user_region: Option<&str>,
) -> Result<VideoAvailability, Box<dyn std::error::Error>> {
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?id={}&key={}&part=status,contentDetails",
        video_id, api_key
    );
    
    let response: VideoListResponse = reqwest::get(&url)
        .await?
        .json()
        .await?;
    
    // Check if video exists
    if response.items.is_empty() {
        return Ok(VideoAvailability {
            exists: false,
            is_embeddable: false,
            is_public: false,
            is_region_blocked: false,
            is_age_restricted: false,
            upload_status: "not_found".to_string(),
            unavailability_reasons: vec!["Video not found, deleted, or private".to_string()],
        });
    }
    
    let video = &response.items[0];
    let mut reasons: Vec<String> = Vec::new();
    
    // Extract status information
    let status = video.status.as_ref();
    let content_details = video.content_details.as_ref();
    
    // Check privacy status
    let privacy_status = status
        .map(|s| s.privacy_status.as_str())
        .unwrap_or("unknown");
    let is_public = privacy_status == "public";
    
    if privacy_status == "private" {
        reasons.push("Video is private".to_string());
    } else if privacy_status == "unlisted" {
        reasons.push("Video is unlisted (accessible but not searchable)".to_string());
    }
    
    // Check embeddability
    let is_embeddable = status
        .map(|s| s.embeddable)
        .unwrap_or(false);
    
    if !is_embeddable {
        reasons.push("Video embedding is disabled".to_string());
    }
    
    // Check upload status
    let upload_status = status
        .map(|s| s.upload_status.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    match upload_status.as_str() {
        "deleted" => reasons.push("Video has been deleted".to_string()),
        "failed" => reasons.push("Video processing failed".to_string()),
        "rejected" => reasons.push("Video was rejected by YouTube".to_string()),
        "uploaded" => reasons.push("Video is still processing".to_string()),
        _ => {}
    }
    
    // Check region restrictions
    let mut is_region_blocked = false;
    if let Some(region) = user_region {
        if let Some(details) = content_details {
            if let Some(restriction) = &details.region_restriction {
                if let Some(allowed) = &restriction.allowed {
                    is_region_blocked = !allowed.contains(&region.to_string());
                    if is_region_blocked {
                        reasons.push(format!("Video not available in region: {}", region));
                    }
                } else if let Some(blocked) = &restriction.blocked {
                    is_region_blocked = blocked.contains(&region.to_string());
                    if is_region_blocked {
                        reasons.push(format!("Video blocked in region: {}", region));
                    }
                }
            }
        }
    }
    
    // Check age restriction
    let is_age_restricted = content_details
        .and_then(|cd| cd.content_rating.as_ref())
        .and_then(|cr| cr.yt_rating.as_ref())
        .map(|rating| rating == "ytAgeRestricted")
        .unwrap_or(false);
    
    if is_age_restricted {
        reasons.push("Video is age-restricted".to_string());
    }
    
    Ok(VideoAvailability {
        exists: true,
        is_embeddable,
        is_public,
        is_region_blocked,
        is_age_restricted,
        upload_status,
        unavailability_reasons: reasons,
    })
}


// Batch Complete Check

pub async fn check_multiple_videos(
    video_ids: &[String],
    api_key: &str,
) -> Result<Vec<(String, VideoAvailability)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    // Process in batches of 50
    for chunk in video_ids.chunks(50) {
        let ids_param = chunk.join(",");
        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?id={}&key={}&part=status,contentDetails",
            ids_param, api_key
        );
        
        let response: VideoListResponse = reqwest::get(&url)
            .await?
            .json()
            .await?;
        
        // Create a set of returned video IDs
        let returned_ids: HashSet<String> = response.items
            .iter()
            .map(|v| v.id.clone())
            .collect();
        
        // Check each requested ID
        for video_id in chunk {
            if !returned_ids.contains(video_id) {
                // Video not found
                results.push((
                    video_id.clone(),
                    VideoAvailability {
                        exists: false,
                        is_embeddable: false,
                        is_public: false,
                        is_region_blocked: false,
                        is_age_restricted: false,
                        upload_status: "not_found".to_string(),
                        unavailability_reasons: vec!["Video not found or deleted".to_string()],
                    }
                ));
            } else {
                // Process available video (extract from response.items)
                let video = response.items
                    .iter()
                    .find(|v| v.id == *video_id)
                    .unwrap();
                
                // Use similar logic as check_video_availability
                results.push((video_id.clone(), process_video_availability(video)));
            }
        }
    }
    
    Ok(results)
}

fn process_video_availability(video: &Video) -> VideoAvailability {
    // Similar logic to the main function
    let mut reasons = Vec::new();
    
    let is_embeddable = video.status
        .as_ref()
        .map(|s| s.embeddable)
        .unwrap_or(false);
    
    if !is_embeddable {
        reasons.push("Embedding disabled".to_string());
    }
    
    // ... (rest of checking logic)
    
    VideoAvailability {
        exists: true,
        is_embeddable,
        is_public: video.status.as_ref().map(|s| s.privacy_status == "public").unwrap_or(false),
        is_region_blocked: false,  // Would need region parameter
        is_age_restricted: false,  // Check content_details
        upload_status: video.status.as_ref().map(|s| s.upload_status.clone()).unwrap_or_default(),
        unavailability_reasons: reasons,
    }
}