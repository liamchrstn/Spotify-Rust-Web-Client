export function loginWithSpotify() {
    // Redirect to the server's login endpoint instead of Spotify's authorize URL
    window.location.href = 'http://localhost:8000/login';
}

export function getAccessToken() {
    // Fetch the access token from the server
    return fetch('http://localhost:8000/get_token')
        .then(response => response.json())
        .then(data => {
            if (data.access_token) {
                localStorage.setItem('spotify_token', data.access_token);
                return data.access_token;
            }
            return null;
        })
        .catch(() => null);
}

// Attach loginWithSpotify to the window object
window.loginWithSpotify = loginWithSpotify;
