import { set_sdk_status } from './spotify_egui.js';

console.log('playback.js loaded'); // Added initial load log

// Define the callback before loading the SDK
window.onSpotifyWebPlaybackSDKReady = () => {
    console.log('Spotify Web Playback SDK is ready');
    initializePlayer();
};

function initializePlayer() {
    const token = localStorage.getItem('spotify_token');
    if (!token) {
        console.warn('No token available for player initialization');
        set_sdk_status('No Token');
        return;
    }

    const player = new Spotify.Player({
        name: 'Web Playback SDK Quick Start Player',
        getOAuthToken: cb => { cb(token); }
    });

    // Error handling
    player.addListener('initialization_error', ({ message }) => {
        console.error(message);
        set_sdk_status('Init Error');
    });

    player.addListener('authentication_error', ({ message }) => {
        console.error(message);
        set_sdk_status('Auth Error');
    });

    player.addListener('account_error', ({ message }) => { console.error(message); });
    player.addListener('playback_error', ({ message }) => { console.error(message); });

    // Playback status updates
    player.addListener('player_state_changed', state => { console.log('Player state changed:', state); });

    // Ready
    player.addListener('ready', ({ device_id }) => {
        console.log('Ready with Device ID', device_id);
        set_sdk_status('Ready');
        console.log('SDK status set to Ready'); // Log after setting status
    });

    // Not Ready
    player.addListener('not_ready', ({ device_id }) => {
        console.log('Device ID has gone offline', device_id);
        set_sdk_status('Not Ready');
        console.log('SDK status set to Not Ready'); // Log after setting status
    });

    // Connect to the player!
    player.connect()
        .then(success => {
            if (success) {
                console.log('Successfully connected to Spotify Player');
                set_sdk_status('Connected');
            } else {
                console.log('Failed to connect to Spotify Player');
                set_sdk_status('Connection Failed');
            }
        })
        .catch(error => {
            console.error('Error connecting to Spotify Player:', error);
            set_sdk_status('Connection Error');
        });
}

// Export for use in other modules
export { initializePlayer };
