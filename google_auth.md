# Building Google Calendar OAuth for a Desktop App

I recently added Google Calendar integration to my Tauri desktop app, and honestly, it was more interesting than I expected. Here's how it went down.

## The Problem

I wanted users to connect their Google Calendar so my task manager could sync events. Simple enough, right? Well, desktop OAuth is different from web OAuth. There's no reliable callback URL because you're not running on a domain.

## First Attempt: Device Code Flow

My initial plan was to use Google's Device Code Flow. You know, that thing where you show users a code and tell them to visit `google.com/device` and enter it? Yeah, that one.

Seemed perfect for a desktop app. No need for localhost servers, no redirect URIs to configure. Just show a code, poll for authorization, done.

**Spoiler**: It wasn't that simple.

Turns out Device Code Flow has some annoying limitations:
- Needs a specific OAuth client type ("TVs and Limited Input devices")
- That client type doesn't support all the scopes I needed
- Google kept throwing `invalid_client` errors even with the right setup
- The polling pattern felt clunky

After a few hours of debugging and reading Google's OAuth docs (which, let's be honest, could be clearer), I gave up on this approach.

## The Better Way: Localhost Redirect

Then I discovered what VS Code, Spotify, and other desktop apps actually do: spin up a tiny HTTP server on localhost, redirect there, and shut it down immediately after.

I found [this excellent article](https://codevoweb.com/how-to-implement-google-oauth2-in-rust/) that walks through implementing Google OAuth2 in Rust, and it clicked. This localhost approach is actually the standard pattern, and for good reason:
- Works instantly (no polling)
- Better user experience
- Simpler error handling
- Desktop OAuth clients automatically accept any localhost port

The flow goes like this:

1. User clicks "Connect Calendar"
2. App opens browser → Google's OAuth page
3. User authorizes
4. Browser redirects to `http://localhost:3333/oauth/callback?code=...`
5. My tiny HTTP server catches the code
6. Exchange code for tokens
7. Show pretty success page
8. Done!

## Implementation

### The OAuth Handler

The core is in `google_oauth.rs`. It does everything:

```rust
pub async fn start_oauth_flow() -> Result<CalendarCredentials, String>
```

When you call this function, it:
- Generates a random "state" parameter (CSRF protection - important!)
- Opens the user's browser with the Google authorization URL
- Starts a tiny HTTP server on port 3333
- Waits for the callback (with a 5-minute timeout)
- Validates the state to prevent CSRF attacks
- Exchanges the authorization code for access and refresh tokens
- Grabs the user's email
- Returns everything wrapped in a nice struct

**Why port 3333?** Because port 8080 is usually taken by dev servers, and 3333 is less common. Desktop OAuth clients don't care what port you use - they accept any localhost port automatically.

### The Architecture

I kept things clean with three layers:

**Thirdparty Layer** (`thirdparty/calendar/google_oauth.rs`)
Talks to Google's API. Makes HTTP requests, parses responses, handles OAuth. Doesn't know anything about databases or app logic.

**Service Layer** (`services/calendar_service.rs`)
Orchestrates everything. Calls the thirdparty layer, saves to database, handles business logic. This is the glue.

**Commands Layer** (`commands/calendar_commands.rs`)
Tauri commands that the frontend calls. Super thin - just bridges JavaScript to Rust.

**Database Layer** (`db/mod.rs`)
All SQL operations. Separate SQL files loaded at compile time with `include_str!`. No raw SQL strings in Rust code.

This separation made everything easier to test and maintain. Each layer has one job.

### Security Stuff

**CSRF Protection**: Every OAuth request gets a random 32-character state parameter. When the callback comes back, we verify it matches. If not, someone's trying something fishy.

**Token Storage**: Access tokens expire after an hour. Refresh tokens last longer and live in SQLite. In production, you'd want to encrypt these or use the OS keychain, but for now, SQLite works.

**Timeouts**: The callback server only waits 5 minutes. After that, it gives up and tells the user to try again.

### The Callback Pages

Initially, I had inline HTML in the Rust code. Looked terrible. Like 40 lines of HTML string in the middle of my OAuth logic.

So I moved them to separate files:
- `oauth_pages/success.html` - Pretty success page
- `oauth_pages/error.html` - Error page with the actual error message
- `oauth_pages/security_error.html` - CSRF attack warning

These load at compile time with `include_str!`, so there's zero runtime overhead. And now I can edit the HTML without touching Rust code. Much cleaner.

I styled them to match the app's design - same calming teal colors, soft shadows, clean typography. Makes the whole flow feel cohesive.

## Google Cloud Setup

Setting this up is straightforward:

1. Go to Google Cloud Console
2. Create an OAuth 2.0 Client ID
3. Choose **"Desktop app"** (not "Web application")
4. Copy the Client ID and Client Secret
5. Enable Google Calendar API
6. Add scopes to OAuth Consent Screen:
   - `calendar.events` for reading/writing events
   - `userinfo.email` to show which account is connected

Desktop apps don't need to configure redirect URIs - Google automatically allows all localhost URLs. This is different from web apps where you have to whitelist specific URLs.

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

## What's Next

This implementation handles the OAuth flow, but I still need to:
- Add automatic token refresh (access tokens expire hourly)
- Actually sync calendar events (right now it just connects)
- Store tokens in OS keychain instead of SQLite

But the foundation is solid. The OAuth flow works reliably, the code is clean, and users can connect their calendars without any friction.

## The Code

If you want to see how this all fits together, the main files are:

- `src-tauri/src/thirdparty/calendar/google_oauth.rs` - OAuth implementation
- `src-tauri/src/services/calendar_service.rs` - Business logic
- `src-tauri/src/commands/calendar_commands.rs` - Tauri commands
- `src-tauri/src/oauth_pages/*.html` - Callback pages

---