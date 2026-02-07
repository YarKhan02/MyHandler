use crate::structs::calendar::CalendarCredentials;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tiny_http::{Server, Response};

// OAuth Configuration - Replace these with your Google Cloud credentials
const CLIENT_ID: &str = "YOUR_CLIENT_ID.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "YOUR_CLIENT_SECRET";
const REDIRECT_URI: &str = "http://localhost:3333/oauth/callback";
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const SCOPES: &str = "https://www.googleapis.com/auth/calendar.events https://www.googleapis.com/auth/userinfo.email";

// Load HTML templates at compile time
const SUCCESS_HTML: &str = include_str!("../../oauth_pages/success.html");
const ERROR_HTML: &str = include_str!("../../oauth_pages/error.html");
const SECURITY_ERROR_HTML: &str = include_str!("../../oauth_pages/security_error.html");

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    #[allow(dead_code)]
    token_type: String,
}

#[derive(Deserialize)]
struct UserInfo {
    email: String,
}

// Generate random state for CSRF protection
fn generate_state() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub async fn start_oauth_flow() -> Result<CalendarCredentials, String> {
    // Generate auth URL with state
    let state = generate_state();
    
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
        GOOGLE_AUTH_URL,
        urlencoding::encode(CLIENT_ID),
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPES),
        state
    );
    
    // Open browser
    webbrowser::open(&auth_url)
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    
    // Start local server to receive callback
    let server = Server::http("127.0.0.1:3333")
        .map_err(|e| format!("Failed to start server: {}", e))?;
    
    let code_result = Arc::new(Mutex::new(None::<String>));
    let code_clone = code_result.clone();
    
    // Wait for callback with timeout
    let timeout = std::time::Duration::from_secs(300); // 5 minutes
    let start = std::time::Instant::now();
    
    for request in server.incoming_requests() {
        if start.elapsed() > timeout {
            return Err("Authorization timeout after 5 minutes".to_string());
        }
        
        let url = request.url().to_string();
        
        // Only handle the callback path
        if !url.starts_with("/oauth/callback") {
            let _ = request.respond(Response::from_string("Not found").with_status_code(404));
            continue;
        }
        
        // Parse query parameters
        if let Some(query) = url.split('?').nth(1) {
            let mut code = None;
            let mut received_state = None;
            let mut error = None;
            
            for param in query.split('&') {
                let parts: Vec<&str> = param.split('=').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "code" => code = Some(urlencoding::decode(parts[1]).unwrap_or_default().to_string()),
                        "state" => received_state = Some(parts[1].to_string()),
                        "error" => error = Some(parts[1].to_string()),
                        _ => {}
                    }
                }
            }
            
            // Check for error
            if let Some(err) = error {
                let html = ERROR_HTML.replace("{{ERROR_MESSAGE}}", &err);
                let response = Response::from_string(html)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                return Err(format!("Authorization error: {}", err));
            }
            
            // Verify state (CSRF protection)
            if received_state.as_deref() != Some(&state) {
                let response = Response::from_string(SECURITY_ERROR_HTML)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                return Err("Invalid state - possible CSRF attack".to_string());
            }
            
            if let Some(auth_code) = code {
                println!("Authorization code received!");
                
                // Send success page to browser
                let response = Response::from_string(SUCCESS_HTML)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                
                *code_clone.lock().unwrap() = Some(auth_code);
                break;
            }
        }
    }
    
    let auth_code = code_result.lock().unwrap().take()
        .ok_or_else(|| "No authorization code received".to_string())?;
    
    // Exchange code for tokens
    exchange_code_for_tokens(&auth_code).await
}

async fn exchange_code_for_tokens(code: &str) -> Result<CalendarCredentials, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("code", code),
        ("grant_type", "authorization_code"),
        ("redirect_uri", REDIRECT_URI),
    ];
    
    let response = client
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to exchange code for tokens: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        eprintln!("Token exchange failed: {} - {}", status, error_body);
        return Err(format!("Token exchange failed: {}", status));
    }
    
    let token_data: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;
    
    let refresh_token = token_data.refresh_token
        .ok_or_else(|| "No refresh token received. Try revoking app access and reconnecting.".to_string())?;
    
    let token_expiry = Utc::now() + Duration::seconds(token_data.expires_in);
    
    // Get user email
    let email = get_user_email(&token_data.access_token).await?;
    
    Ok(CalendarCredentials {
        email,
        access_token: token_data.access_token,
        refresh_token,
        token_expiry,
    })
}

async fn get_user_email(access_token: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Failed to get user info: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to get user email: {}", response.status()));
    }
    
    let user_info: UserInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;
    
    Ok(user_info.email)
}

pub async fn refresh_access_token(refresh_token: &str) -> Result<(String, i64), String> {
    println!("Refreshing access token...");
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    
    let params = [
        ("client_id", CLIENT_ID),
        ("client_secret", CLIENT_SECRET),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    
    let response = client
        .post(GOOGLE_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {} - {}", status, error_body));
    }
    
    let token_data: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;
    
    println!("Token refreshed successfully");
    Ok((token_data.access_token, token_data.expires_in))
}
