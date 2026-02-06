UPDATE calendar_credentials 
SET email = ?, 
    access_token = ?, 
    refresh_token = ?, 
    token_expiry = ?, 
    updated_at = CURRENT_TIMESTAMP 
WHERE id = 1
