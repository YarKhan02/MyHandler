# Building Google Calendar OAuth for a Desktop App

I recently added Google Calendar integration to my Tauri desktop app, and honestly, it was more interesting than I expected. OAuth in desktop apps is a different beast than web apps - there's no domain, no guaranteed redirect URL, and you're running entirely on the user's machine.

Here's the story of how I got it working, complete with code and the lessons learned.

## The Challenge

I wanted users to connect their Google Calendar so my task manager could:
- Create calendar events for tasks with deadlines
- Update events when tasks change
- Delete events when tasks are completed
- Sync reminder frequencies

Simple enough, right? Well...

### Attempt 1: Device Code Flow (Failed)

My initial plan was Google's Device Code Flow - that TV-style flow where you show users a code and tell them to visit `google.com/device`. Seemed perfect:
- No localhost server needed
- Works on any device
- User-friendly for limited-input devices

But here's what went wrong:

```rust
// Device Flow attempt - didn't work!
let response = client
    .post("https://oauth2.googleapis.com/device/code")
    .form(&[("client_id", CLIENT_ID), ("scope", SCOPES)])
    .send()
    .await?;

// Error: 401 Unauthorized
// "error": "invalid_client",
// "error_description": "Invalid client type."
```

The issues:
1. **Wrong credential type**: Device Flow requires "TVs and Limited Input devices" OAuth client type
2. **Desktop app credentials don't work**: My "Desktop app" credentials were rejected
3. **Limited scope support**: Some Google APIs don't support Device Flow  
4. **Poor UX**: Copy-pasting codes feels clunky for a desktop app

After hitting walls with `invalid_client` errors for hours, I realized this wasn't the right approach for a desktop app with full keyboard/browser access.

### Attempt 2: Localhost Redirect (Success!)

Then I discovered what VS Code, Spotify, Slack, and virtually every other desktop app does: **spin up a temporary HTTP server on localhost**.

The flow:
1. Open user's browser → Google OAuth page
2. User authorizes
3. Google redirects to `http://localhost:3333/oauth/callback?code=...`
4. Your tiny HTTP server catches it
5. Exchange code for tokens
6. Show success page
7. Shut down server

This is actually the **standard OAuth 2.0 Authorization Code flow**, adapted for desktop apps.

**Why it works:**
- Google's "Desktop app" credentials automatically accept any localhost redirect
- No need to pre-configure ports or URLs
- Instant response (no polling)
- Familiar user experience
- Supports all Google API scopes

I found [this excellent Rust OAuth guide](https://codevoweb.com/how-to-implement-google-oauth2-in-rust/) that confirmed this is the right path. Let's build it.

## The Complete Implementation

### Project Structure

First, let's look at how everything is organized:

```
src-tauri/
├── src/
│   ├── commands/
│   │   └── calendar_commands.rs      # Tauri commands (Frontend ↔ Rust)
│   ├── services/
│   │   └── calendar_service.rs       # Business logic layer
│   ├── thirdparty/
│   │   └── calendar/
│   │       └── google_oauth.rs       # Google API integration
│   ├── structs/
│   │   └── calendar/
│   │       └── calendar_credentials.rs  # Data structures
│   ├── db/
│   │   ├── mod.rs                    # Database operations
│   │   ├── tables/
│   │   │   └── calendar_credentials.sql
│   │   └── sql/
│   │       ├── save_calendar_credentials.sql
│   │       └── get_calendar_credentials.sql
│   └── oauth_pages/
│       ├── success.html              # OAuth success page
│       ├── error.html                # OAuth error page
│       └── security_error.html       # CSRF attack warning
```

This layered approach keeps concerns separated:
- **Commands**: Thin layer exposing Rust functions to JavaScript
- **Services**: Orchestrates business logic and database operations
- **Thirdparty**: Talks to external APIs (Google)
- **Database**: All SQL isolated in separate files
- **Structs**: Shared data types

### Step 1: Data Structures

Start with the core data type for storing OAuth credentials:

```rust
// src/structs/calendar/calendar_credentials.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarCredentials {
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry: DateTime<Utc>,
}
```

**Why this structure?**
- `email`: Shows user which account is connected
- `access_token`: Short-lived (1 hour), used for API calls
- `refresh_token`: Long-lived, gets new access tokens
- `token_expiry`: When to refresh the access token
- `DateTime<Utc>`: Timezone-aware timestamps prevent bugs

### Step 2: Database Schema

Single-user table with a clever constraint:

```sql
-- src/db/tables/calendar_credentials.sql
CREATE TABLE IF NOT EXISTS calendar_credentials (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Only allows ONE row
    email VARCHAR(255) NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    token_expiry DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Initialize with empty placeholder
INSERT OR IGNORE INTO calendar_credentials (id, email, access_token, refresh_token, token_expiry)
VALUES (1, '', '', '', CURRENT_TIMESTAMP);
```

**The `CHECK (id = 1)` trick** ensures only one row can exist. This is a single-user desktop app, so we don't need multiple accounts (yet). Trying to insert `id = 2` will fail.

Save credentials:

```sql
-- src/db/sql/save_calendar_credentials.sql
UPDATE calendar_credentials 
SET email = ?, 
    access_token = ?, 
    refresh_token = ?, 
    token_expiry = ?, 
    updated_at = CURRENT_TIMESTAMP 
WHERE id = 1
```

Get credentials:

```sql
-- src/db/sql/get_calendar_credentials.sql
SELECT email, access_token, refresh_token, token_expiry
FROM calendar_credentials
WHERE id = 1 AND email != ''
```

The `email != ''` check returns `NULL` if never authenticated.

### Step 3: The OAuth Flow (The Heart of It All)

Here's the complete OAuth implementation:

```rust
// src/thirdparty/calendar/google_oauth.rs
use crate::structs::calendar::CalendarCredentials;
use chrono::{Duration, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tiny_http::{Server, Response};

// OAuth Configuration
const CLIENT_ID: &str = "your-client-id.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "your-client-secret";
const REDIRECT_URI: &str = "http://localhost:3333/oauth/callback";
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const SCOPES: &str = "https://www.googleapis.com/auth/calendar.events https://www.googleapis.com/auth/userinfo.email";

// Load HTML templates at compile time (zero runtime cost!)
const SUCCESS_HTML: &str = include_str!("../../oauth_pages/success.html");
const ERROR_HTML: &str = include_str!("../../oauth_pages/error.html");
const SECURITY_ERROR_HTML: &str = include_str!("../../oauth_pages/security_error.html");

// Response structures for deserializing Google's JSON
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
    // Step 1: Generate CSRF protection token
    let state = generate_state();
    
    // Step 2: Build authorization URL
    let auth_url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
        GOOGLE_AUTH_URL,
        urlencoding::encode(CLIENT_ID),
        urlencoding::encode(REDIRECT_URI),
        urlencoding::encode(SCOPES),
        state
    );
    
    // Step 3: Open browser to Google's OAuth page
    webbrowser::open(&auth_url)
        .map_err(|e| format!("Failed to open browser: {}", e))?;
    
    // Step 4: Start tiny HTTP server for callback
    let server = Server::http("127.0.0.1:3333")
        .map_err(|e| format!("Failed to start server: {}", e))?;
    
    // Shared state for the authorization code
    let code_result = Arc::new(Mutex::new(None::<String>));
    let code_clone = code_result.clone();
    
    // Step 5: Wait for callback (with 5-minute timeout)
    let timeout = std::time::Duration::from_secs(300);
    let start = std::time::Instant::now();
    
    for request in server.incoming_requests() {
        if start.elapsed() > timeout {
            return Err("Authorization timeout after 5 minutes".to_string());
        }
        
        let url = request.url().to_string();
        
        // Only handle our callback path
        if !url.starts_with("/oauth/callback") {
            let _ = request.respond(Response::from_string("Not found").with_status_code(404));
            continue;
        }
        
        // Step 6: Parse query parameters
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
            
            // Step 7: Handle errors from Google
            if let Some(err) = error {
                let html = ERROR_HTML.replace("{{ERROR_MESSAGE}}", &err);
                let response = Response::from_string(html)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                return Err(format!("Authorization error: {}", err));
            }
            
            // Step 8: VERIFY STATE (Critical security check!)
            if received_state.as_deref() != Some(&state) {
                let response = Response::from_string(SECURITY_ERROR_HTML)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                return Err("Invalid state - possible CSRF attack".to_string());
            }
            
            // Step 9: Success! Send pretty page to browser
            if let Some(auth_code) = code {
                let response = Response::from_string(SUCCESS_HTML)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                let _ = request.respond(response);
                
                *code_clone.lock().unwrap() = Some(auth_code);
                break;
            }
        }
    }
    
    // Step 10: Extract the authorization code
    let auth_code = code_result.lock().unwrap().take()
        .ok_or_else(|| "No authorization code received".to_string())?;
    
    // Step 11: Exchange code for tokens
    exchange_code_for_tokens(&auth_code).await
}
```

**Why port 3333?**  
Ports 8080/5173 are often taken by dev servers. Port 3333 is uncommon enough to usually be free. Google's "Desktop app" credentials accept **any** localhost port automatically - no configuration needed.

**The CSRF state parameter:**  
This prevents attackers from tricking users into authorizing malicious apps. The flow:
1. App generates random 32-char string
2. Includes it in authorization URL
3. Google includes it in callback
4. App verifies they match
5. If mismatch → Attack attempt detected!

**The CSRF state parameter:**  
This prevents attackers from tricking users into authorizing malicious apps. The flow:
1. App generates random 32-char string
2. Includes it in authorization URL
3. Google includes it in callback
4. App verifies they match
5. If mismatch → Attack attempt detected!

### Step 4: Token Exchange

After getting the authorization code, exchange it for actual tokens:

```rust
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
    
    // Get user's email for display
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
```

**Why we need both tokens:**
- **Access token**: Short-lived (1 hour), used for API calls
- **Refresh token**: Long-lived (months/years), gets new access tokens

**Getting the email:**  
We call Google's userinfo endpoint to show users which account is connected. This helps when they have multiple Google accounts.

### Step 5: Token Refresh

Access tokens expire every hour. Here's how to refresh them:

```rust
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
```

This function returns a tuple `(new_access_token, expires_in_seconds)`. The caller saves the new token to the database and updates the expiry timestamp.

This function returns a tuple `(new_access_token, expires_in_seconds)`. The caller saves the new token to the database and updates the expiry timestamp.

### Step 6: Service Layer (Business Logic)

The service layer orchestrates OAuth flow and database operations:

```rust
// src/services/calendar_service.rs
use crate::db::{self, Database};
use crate::structs::calendar::CalendarCredentials;
use crate::thirdparty::calendar;
use chrono::{DateTime, Utc, Duration};

pub async fn start_oauth_flow(db: &Database) -> Result<CalendarCredentials, String> {
    // Start OAuth flow and get credentials
    let credentials = calendar::start_oauth_flow().await?;
    
    // Save to database
    save_credentials(db, &credentials)?;
    
    Ok(credentials)
}

pub fn save_credentials(db: &Database, creds: &CalendarCredentials) -> Result<(), String> {
    let conn = db.get_connection();
    db::save_calendar_credentials(&conn, creds)
        .map_err(|e| format!("Failed to save credentials: {}", e))
}

pub fn get_credentials(db: &Database) -> Result<Option<CalendarCredentials>, String> {
    let conn = db.get_connection();
    db::get_calendar_credentials(&conn)
        .map_err(|e| format!("Failed to get credentials: {}", e))
}

pub fn disconnect_calendar(db: &Database) -> Result<(), String> {
    let conn = db.get_connection();
    db::clear_calendar_credentials(&conn)
        .map_err(|e| format!("Failed to disconnect calendar: {}", e))
}

// Get valid access token, refreshing if needed
pub async fn get_valid_access_token(db: &Database) -> Result<String, String> {
    let mut creds = get_credentials(db)?
        .ok_or_else(|| "No calendar credentials found".to_string())?;
    
    // Check if token is expired (with 5-minute buffer)
    let now = Utc::now();
    let buffer = Duration::minutes(5);
    
    if now + buffer >= creds.token_expiry {
        println!("Access token expired, refreshing...");
        
        // Refresh the token
        let (new_access_token, expires_in) = calendar::refresh_access_token(&creds.refresh_token).await?;
        
        // Update credentials
        creds.access_token = new_access_token;
        creds.token_expiry = now + Duration::seconds(expires_in);
        
        // Save to database
        save_credentials(db, &creds)?;
    }
    
    Ok(creds.access_token)
}
```

**The `get_valid_access_token` function is clever:**
1. Checks if token expires in next 5 minutes
2. If expiring soon → Refreshes automatically
3. Saves new token to database
4. Returns valid token

This means other parts of the app can just call this function and always get a working token. No need to handle expiry everywhere.

### Step 7: Tauri Commands (Frontend Bridge)

Expose Rust functions to JavaScript:

```rust
// src/commands/calendar_commands.rs
use tauri::State;
use crate::db;
use crate::services::calendar_service;
use crate::structs::calendar::CalendarCredentials;

#[tauri::command]
pub async fn start_calendar_auth(db: State<'_, db::Database>) -> Result<CalendarCredentials, String> {
    calendar_service::start_oauth_flow(&db).await
}

#[tauri::command]
pub fn get_calendar_status(db: State<'_, db::Database>) -> Result<Option<CalendarCredentials>, String> {
    calendar_service::get_credentials(&db)
}

#[tauri::command]
pub fn disconnect_calendar(db: State<'_, db::Database>) -> Result<(), String> {
    calendar_service::disconnect_calendar(&db)
}
```

**These commands are thin wrappers** - they just bridge JavaScript to Rust. All the real logic is in the service layer.

Don't forget to register them in `main.rs`:

```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_calendar_auth,
            get_calendar_status,
            disconnect_calendar,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Step 8: Frontend Integration (React/TypeScript)

Finally, the UI layer:

```typescript
// In your Tauri commands wrapper
import { invoke } from '@tauri-apps/api/core';

export const tauriCommands = {
  startCalendarAuth: async (): Promise<CalendarCredentials> => {
    return await invoke<CalendarCredentials>('start_calendar_auth');
  },
  
  getCalendarStatus: async (): Promise<CalendarCredentials | null> => {
    return await invoke<CalendarCredentials | null>('get_calendar_status');
  },
  
  disconnectCalendar: async (): Promise<void> => {
    await invoke('disconnect_calendar');
  },
};
```

And in your React component:

```tsx
// src/pages/SettingsPage.tsx
const [settings, setSettings] = useState<Settings | null>(null);
const [connecting, setConnecting] = useState(false);

const startCalendarConnection = async () => {
  setConnecting(true);
  try {
    const credentials = await tauriCommands.startCalendarAuth();
    console.log('Calendar connected:', credentials);
    
    // Refresh settings to show connected status
    const updatedSettings = await tauriCommands.getSettings();
    setSettings(updatedSettings);
  } catch (error) {
    console.error('Failed to start calendar auth:', error);
  } finally {
    setConnecting(false);
  }
};

const disconnectCalendar = async () => {
  try {
    await tauriCommands.disconnectCalendar();
    const updatedSettings = await tauriCommands.getSettings();
    setSettings(updatedSettings);
  } catch (error) {
    console.error('Failed to disconnect calendar:', error);
  }
};

// In your JSX:
{settings.calendarIntegrationEnabled ? (
  <div>
    <Label>Connected: {settings.calendarEmail}</Label>
    <Button onClick={disconnectCalendar}>Disconnect</Button>
  </div>
) : (
  <Button onClick={startCalendarConnection} disabled={connecting}>
    {connecting ? 'Connecting...' : 'Connect Calendar'}
  </Button>
)}
```

**User experience:**
1. User clicks "Connect Calendar"
2. Browser opens to Google (automatic)
3. User authorizes
4. Success page shows in browser
5. User closes browser tab
6. App shows "Connected: user@gmail.com"

Total time: ~10 seconds.

### Step 9: OAuth Callback HTML Pages

When OAuth completes, the user needs feedback. I created three HTML pages that display in the browser:

**Success Page** (`src/oauth_pages/success.html`) - Shows when authorization succeeds:
```html
<!DOCTYPE html>
<html>
<head>
    <title>Authorization Successful</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif;
               display: flex; justify-content: center; align-items: center;
               min-height: 100vh; background: hsl(40, 20%, 98%); }
        .container { background: white; padding: 48px; border-radius: 16px;
                     box-shadow: 0 4px 20px rgba(0,0,0,0.08); text-align: center; }
        .icon { width: 64px; height: 64px; margin: 0 auto 24px;
                background: hsl(175, 40%, 94%); border-radius: 50%; }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">✓</div>
        <h1>Authorization Successful!</h1>
        <p>Your Google Calendar has been connected to MyHandler.</p>
        <p>You can now close this window.</p>
    </div>
</body>
</html>
```

**Error Page** (`src/oauth_pages/error.html`) - Shows when something goes wrong:
```html
<!DOCTYPE html>
<html>
<head>
    <title>Authorization Failed</title>
    <style>
        .icon { background: hsl(0, 70%, 95%); color: hsl(0, 72%, 58%); }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">✗</div>
        <h1>Authorization Failed</h1>
        <p>{{ERROR_MESSAGE}}</p>
        <p>Please close this window and try again.</p>
    </div>
</body>
</html>
```

**Security Error Page** (`src/oauth_pages/security_error.html`) - CSRF protection triggered:
```html
<!DOCTYPE html>
<html>
<head>
    <title>Security Error</title>
    <style>
        .icon { background: hsl(40, 70%, 95%); color: hsl(40, 90%, 50%); }
    </style>
</head>
<body>
    <div class="container">
        <div class="icon">⚠</div>
        <h1>Security Error</h1>
        <p>Invalid state parameter. This could be a CSRF attack.</p>
        <p>For your security, authorization stopped.</p>
    </div>
</body>
</html>
```

**Embed them at compile time:**

In `google_oauth.rs`:
```rust
const SUCCESS_HTML: &str = include_str!("../oauth_pages/success.html");
const ERROR_HTML: &str = include_str!("../oauth_pages/error.html");
const SECURITY_ERROR_HTML: &str = include_str!("../oauth_pages/security_error.html");
```

This embeds the HTML directly in the binary - no external files needed!

### Step 10: Google Cloud Console Setup

Here's the complete setup process:

**1. Create Google Cloud Project**
- Go to [console.cloud.google.com](https://console.cloud.google.com)
- Click "New Project"
- Name it (e.g., "MyHandler")
- Click "Create"

**2. Enable Google Calendar API**
- In your project, go to "APIs & Services" → "Library"
- Search for "Google Calendar API"
- Click "Enable"

**3. Create OAuth Credentials**
- Go to "APIs & Services" → "Credentials"
- Click "Create Credentials" → "OAuth 2.0 Client ID"
- **Choose "Desktop app"** (NOT "Web application" - this is critical!)
- Give it a name: "MyHandler Desktop Client"
- Click "Create"
- **Copy the Client ID and Client Secret** - you'll need these

**4. Configure OAuth Consent Screen**
- Go to "APIs & Services" → "OAuth consent screen"
- Choose "External" (unless you have Google Workspace)
- Fill in:
  - App name: "MyHandler"
  - User support email: your email
  - App logo: (optional)
  - App domain: your website (optional during development)
  - Developer contact: your email

**5. Add Scopes**
- Click "Add or Remove Scopes"
- Add:
  - `https://www.googleapis.com/auth/calendar.events` (Read/write events)
  - `https://www.googleapis.com/auth/userinfo.email` (Get user email)
- Click "Update"

**6. Add Test Users (Development Mode)**
- Your app starts in "Testing" mode
- Add your email as a test user
- Once ready for public use, click "Publish App"

**Key Insight:** Desktop apps don't need to configure redirect URIs. Google automatically allows **all localhost URLs** for "Desktop app" credentials. This is different from web applications where you must whitelist specific URLs.

### Security Considerations

**CSRF Protection:**
- Random 32-character state parameter prevents authorization interception
- Validated on every callback
- Used once and discarded

**CLIENT_SECRET in Desktop Apps:**
- Yes, it's embedded in the binary
- Yes, users can extract it
- **This is normal and expected** - VS Code, Slack, Spotify, Discord all do this
- Google knows and allows it
- Real security comes from user consent, not secret hiding
- The "secret" is only secret for web apps with server-side code

**Token Security:**
- Access tokens: 1-hour expiry (short-lived)
- Refresh tokens: Long-lived (revocable by user)
- Stored in local SQLite database
- Consider encrypting the database for extra security

**Network Security:**
- All Google API calls use HTTPS
- 30-second timeouts prevent hanging
- Local server binds only to 127.0.0.1 (not accessible from network)

## Database Schema

Simple single-user table:

```sql
CREATE TABLE calendar_credentials (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Only one user
    email VARCHAR(255) NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    token_expiry DATETIME NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

The `CHECK (id = 1)` constraint ensures only one row exists. This is a single-user desktop app, so I don't need to handle multiple accounts (yet).

And the corresponding Rust struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarCredentials {
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry: DateTime<Utc>,
}
```

Simple, clean, and maps directly to the database columns. The `token_expiry` uses `chrono::DateTime<Utc>` for proper timezone handling.

## The User Experience

From the user's perspective:
1. Click "Connect Calendar" button
2. Browser opens to Google
3. Sign in (if needed) and click "Allow"
4. See a nice success page
5. Close browser tab
6. Back in the app, their email shows as connected

The whole thing takes like 10 seconds. No codes to copy, no polling, no waiting. Just works.

## Dependencies

The key crates that made this possible:

```toml
tiny_http = "0.12"      # Lightweight HTTP server
webbrowser = "1.0"      # Opens browser cross-platform  
reqwest = "0.11"        # HTTP client for API calls
serde = "1.0"           # JSON serialization
rand = "0.8"            # Random state generation
urlencoding = "2.1"     # URL encoding/decoding
```

`tiny_http` is particularly nice - it's a simple, synchronous HTTP server.

## Conclusion

### What I Learned

**1. Device Flow Isn't for Desktop Apps**
I started with Google's Device Flow thinking it was designed for desktop apps. Wrong! It requires "TVs and Limited Input devices" credentials. Google's documentation is misleading here - it says "suitable for devices without browsers" but actually means "for devices that literally can't open browsers" (like smart TVs).

**2. Localhost Redirect is the Standard**
After researching how VS Code, Slack, Spotify, and Discord handle OAuth, I found they all use the Authorization Code Flow with localhost redirect. This is the industry standard for desktop apps, and for good reason - it's fast, secure, and provides excellent UX.

**3. CLIENT_SECRET Isn't Really Secret (for Desktop Apps)**
This was surprising at first. In web apps, CLIENT_SECRET must be kept on the server. But in desktop apps, it's embedded in the binary and users can extract it. Google knows this and allows it. The real security comes from:
- User authenticating with Google
- User explicitly granting permission
- Tokens being bound to the user's account
- User can revoke access anytime

**4. CSRF Protection is Critical**
The random state parameter isn't optional - it prevents authorization interception attacks. Generate it randomly, validate it strictly, use it once.

**5. Token Refresh is Automatic**
With a good service layer, token refresh can be completely transparent. The `get_valid_access_token()` function handles expiry checks and automatic refresh, so the rest of the app never has to think about it.

### Why This Approach Works

**Fast:** User clicks → browser opens → authorize → done. ~10 seconds total.

**Secure:** CSRF protection, HTTPS, token refresh, single-user database constraint.

**Reliable:** No polling, no manual code copying, no timeouts (well, 5-minute timeout but that's plenty).

**Cross-Platform:** Works on macOS, Windows, Linux thanks to `webbrowser` crate.

**Maintainable:** Clean layered architecture (commands → services → oauth → db).


```
Frontend (React/TypeScript)
    ↓ (Tauri commands)
Command Layer (calendar_commands.rs)
    ↓
Service Layer (calendar_service.rs)
    ↓
OAuth Layer (google_oauth.rs) ←→ Google APIs
    ↓
Database Layer (db/mod.rs)
    ↓
SQLite (credentials + events)
```

Each layer has a single responsibility. Changes in one layer rarely affect others. Testing is straightforward.