import init, { start, set_access_token } from './spotify_egui.js';
import { getAccessToken } from './auth.js';
import { initializePlayer } from './playback.js';

async function run() {
    try {
        // Initialize the WASM module
        await init();
        
        // Get access token and set it if available
        const accessToken = await getAccessToken();
        if (accessToken) {
            set_access_token(accessToken);
            // Initialize player if SDK is ready
            if (window.Spotify) {
                initializePlayer();
            }
        }

        await start();

        // Hide loading spinner and show canvas
        document.querySelector('.pl').style.display = 'none';
        document.querySelector('canvas').style.display = 'block';
    } catch (error) {
        console.error('Error initializing application:', error);
        document.querySelector('.pl').style.display = 'none';
    }
}

// Start the application when the page loads
window.addEventListener('load', run);