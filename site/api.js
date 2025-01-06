// Import the WebAssembly module and functions
import init, { start, set_access_token } from './pkg/spotify_egui.js';

// Import auth helper from your auth.js file 
import { getAccessToken } from './auth.js';

async function run() {
    // Initialize the WASM module
    await init();
    await start();

    // Get access token and set it if available
    const accessToken = await getAccessToken();
    if (accessToken) {
        set_access_token(accessToken);
    }

    // Hide loading spinner and show canvas
    document.querySelector('.pl').style.display = 'none';
    document.querySelector('canvas').style.display = 'block';
}

// Start the application
run();