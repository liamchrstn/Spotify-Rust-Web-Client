export function loginWithSpotify() {
    const clientId = '75a6782d877a45d9adf93299e1663ad9';
    const redirectUri = 'http://localhost:8000';
    const scope = 'user-read-email user-library-read'; // Ensure user-library-read scope is included
    const url = `https://accounts.spotify.com/authorize?client_id=${clientId}&response_type=token&redirect_uri=${redirectUri}&scope=${scope}`;
    window.location.href = url;
}

export function getAccessToken() {
    const storedToken = localStorage.getItem('spotify_token');
    if (storedToken) {
        return storedToken;
    }

    const hash = window.location.hash.substring(1);
    const params = hash.split('&').reduce((acc, param) => {
        const [key, value] = param.split('=');
        acc[key] = value;
        return acc;
    }, {});

    if (params.access_token) {
        localStorage.setItem('spotify_token', params.access_token);
        window.location.hash = '';
        return params.access_token;
    }

    return null;
}

// Attach loginWithSpotify to the window object
window.loginWithSpotify = loginWithSpotify;
