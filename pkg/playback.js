import { set_sdk_status } from './spotify_egui.js';
import init, * as wasm from './spotify_egui.js';

// Initialize wasm
init();

console.log('playback.js loaded');

// Define the callback before loading the SDK
window.onSpotifyWebPlaybackSDKReady = () => {
    console.log('Spotify Web Playback SDK is ready');
    initializePlayer();
};

// State management and rate limiting
let lastUpdateTime = 0;
let lastUpdateSuccess = true;
const MIN_UPDATE_INTERVAL = 1000; // Minimum 1 second between updates
const RETRY_INTERVAL = 5000; // Wait 5 seconds after a failure before retrying

function startPlaybackUpdates() {
    // Update immediately
    updatePlaybackState();
    // Then update periodically
    setInterval(checkAndUpdatePlayback, 200);
}

function checkAndUpdatePlayback() {
    const now = Date.now();
    const timeSinceLastUpdate = now - lastUpdateTime;
    
    // If last update failed, wait longer before retrying
    const waitTime = lastUpdateSuccess ? MIN_UPDATE_INTERVAL : RETRY_INTERVAL;
    
    if (timeSinceLastUpdate >= waitTime) {
        updatePlaybackState();
    }
}

async function updatePlaybackState() {
    try {
        lastUpdateTime = Date.now();
        await wasm.get_current_playback();
        lastUpdateSuccess = true;
    } catch (error) {
        console.error('Error updating playback state:', error);
        lastUpdateSuccess = false;
    }
}

function initializePlayer() {
    const token = localStorage.getItem('spotify_token');
    if (!token) {
        console.warn('No token available for player initialization');
        set_sdk_status('No Token');
        return;
    }

    // Disconnect existing player if it exists
    if (window.spotifyPlayer) {
        window.spotifyPlayer.disconnect();
    }

    // Get custom player name from localStorage or use default
    const playerName = localStorage.getItem('player_name') || 'Web Playback SDK Quick Start Player';
    console.log('Initializing player with name:', playerName);

    const player = new Spotify.Player({
        name: playerName,
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
    player.addListener('player_state_changed', state => { 
        console.log('Player state changed:', state);
        window.currentPlayerState = state;
    });

    // Ready
    player.addListener('ready', ({ device_id }) => {
        console.log('Ready with Device ID', device_id);
        window.deviceId = device_id;
        window.isReady = true;
        set_sdk_status('Ready');
    });

    // Not Ready
    player.addListener('not_ready', ({ device_id }) => {
        console.log('Device ID has gone offline', device_id);
        set_sdk_status('Not Ready');
    });

    // Connect to the player!
    player.connect()
        .then(async success => {
            if (success) {
                console.log('Successfully connected to Spotify Player');
                set_sdk_status('Connected');
                window.spotifyPlayer = player;
                
                // Wait a short moment before starting updates
                await new Promise(resolve => setTimeout(resolve, 500));
                startPlaybackUpdates();
                
                // Add play/pause functionality
                window.playPause = async () => {
                    if (!window.isReady) {
                        console.error('Player not ready yet');
                        set_sdk_status('Not Ready');
                        return;
                    }
                    
                    console.log('playPause called');
                    const state = await player.getCurrentState();
                    console.log('Current player state:', state);
                    
                    if (state) {
                        if (state.paused) {
                            await player.resume();
                            console.log('Playback resumed');
                            set_sdk_status('Playing');
                        } else {
                            await player.pause();
                            console.log('Playback paused');
                            set_sdk_status('Paused');
                        }
                    } else {
                        // Check if there are no active devices
                        await wasm.has_active_devices();
                        if (!window.hasActiveDevices) {
                            console.log('No active devices found, attempting device activation...');
                            if (window.deviceId) {
                                await wasm.activate_device(window.deviceId);
                                if (window.deviceActivated) {
                                    console.log('Device activated successfully');
                                    set_sdk_status('Ready');
                                    return;
                                }
                            }
                        }
                        console.log('No playback state available');
                        set_sdk_status('No Playback');
                    }
                };
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

// Add reinitialize function
window.reinitializePlayer = () => {
    console.log('Reinitializing player with new settings...');
    initializePlayer();
};

// Export for use in other modules
export { initializePlayer };
