const clientId = '75a6782d877a45d9adf93299e1663ad9';
const redirectUri = 'http://localhost:8000/';

const authEndpoint = 'https://accounts.spotify.com/authorize';
const tokenEndpoint = 'https://accounts.spotify.com/api/token';
const scope = 'user-read-private user-read-email ugc-image-upload user-read-playback-state user-modify-playback-state user-read-currently-playing app-remote-control streaming playlist-read-private playlist-read-collaborative playlist-modify-private playlist-modify-public user-follow-modify user-follow-read user-read-playback-position user-top-read user-read-recently-played user-library-modify user-library-read';

// Token management
const tokenStorage = {
    setAccessToken: (token) => localStorage.setItem('spotify_token', token),
    getAccessToken: () => localStorage.getItem('spotify_token'),
    setRefreshToken: (token) => localStorage.setItem('spotify_refresh_token', token),
    getRefreshToken: () => localStorage.getItem('spotify_refresh_token'),
    setExpiryTime: (time) => localStorage.setItem('spotify_token_expiry', time),
    getExpiryTime: () => localStorage.getItem('spotify_token_expiry'),
    clear: () => {
        localStorage.removeItem('spotify_token');
        localStorage.removeItem('spotify_refresh_token');
        localStorage.removeItem('spotify_token_expiry');
        localStorage.removeItem('code_verifier');
    }
};

// Generate random string for PKCE
function generateRandomString(length) {
    const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    const values = crypto.getRandomValues(new Uint8Array(length));
    return values.reduce((acc, x) => acc + possible[x % possible.length], '');
}

// Generate code challenge for PKCE
async function generateCodeChallenge(codeVerifier) {
    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);
    const digest = await crypto.subtle.digest('SHA-256', data);
    return btoa(String.fromCharCode(...new Uint8Array(digest)))
        .replace(/=/g, '')
        .replace(/\+/g, '-')
        .replace(/\//g, '_');
}

export async function loginWithSpotify() {
    const codeVerifier = generateRandomString(64);
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    
    localStorage.setItem('code_verifier', codeVerifier);
    
    const params = new URLSearchParams({
        client_id: clientId,
        response_type: 'code',
        redirect_uri: redirectUri,
        scope: scope,
        code_challenge_method: 'S256',
        code_challenge: codeChallenge,
    });

    window.location.href = `${authEndpoint}?${params.toString()}`;
}

async function exchangeToken(code) {
    const codeVerifier = localStorage.getItem('code_verifier');
    
    const response = await fetch(tokenEndpoint, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
            client_id: clientId,
            grant_type: 'authorization_code',
            code: code,
            redirect_uri: redirectUri,
            code_verifier: codeVerifier,
        }),
    });

    const data = await response.json();
    if (response.ok) {
        const expiryTime = Date.now() + (data.expires_in * 1000);
        tokenStorage.setAccessToken(data.access_token);
        tokenStorage.setRefreshToken(data.refresh_token);
        tokenStorage.setExpiryTime(expiryTime);
        return data.access_token;
    } else {
        throw new Error(data.error || 'Failed to exchange token');
    }
}

async function refreshAccessToken() {
    const refreshToken = tokenStorage.getRefreshToken();
    if (!refreshToken) return null;

    try {
        const response = await fetch(tokenEndpoint, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            body: new URLSearchParams({
                grant_type: 'refresh_token',
                refresh_token: refreshToken,
                client_id: clientId,
            }),
        });

        const data = await response.json();
        if (response.ok) {
            const expiryTime = Date.now() + (data.expires_in * 1000);
            tokenStorage.setAccessToken(data.access_token);
            tokenStorage.setExpiryTime(expiryTime);
            if (data.refresh_token) {
                tokenStorage.setRefreshToken(data.refresh_token);
            }
            return data.access_token;
        }
    } catch (error) {
        console.error('Error refreshing token:', error);
        tokenStorage.clear();
        return null;
    }
}

export async function getAccessToken() {
    // Check URL for auth code
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    
    if (code) {
        // Clean up URL
        window.history.replaceState({}, document.title, window.location.pathname);
        const token = await exchangeToken(code);
        window.location.href = redirectUri; // Redirect to the main page
        return token;
    }

    // Check existing token
    const currentToken = tokenStorage.getAccessToken();
    const expiryTime = tokenStorage.getExpiryTime();

    if (!currentToken) return null;

    // Refresh token if expired or expiring soon (within 5 minutes)
    if (!expiryTime || Date.now() > (expiryTime - 300000)) {
        return await refreshAccessToken();
    }

    return currentToken;
}

// Attach function to window for global access
window.loginWithSpotify = loginWithSpotify;