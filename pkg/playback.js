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

let lastTickTime = Date.now();

function updateLocalPlaybackTime() {
    if (window.isPlaying) {
        const now = Date.now();
        const elapsed = now - lastTickTime;
        lastTickTime = now;
        
        // Update local time
        window.currentPlaybackTime = (window.currentPlaybackTime || 0) + elapsed;
        
        // Loop back if we reach the end
        if (window.currentPlaybackTime > window.totalDuration) {
            window.currentPlaybackTime = 0;
        }
    }
}

function startPlaybackUpdates() {
    // Update immediately
    updatePlaybackState();
    
    // Update local time more frequently
    setInterval(updateLocalPlaybackTime, 50); // 50ms for smooth updates
    
    // Check for remote updates periodically
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
        const isPlayerWindowOpen = await wasm.is_player_window_open();
        if (!isPlayerWindowOpen) {
            return;
        }

        // Check if we're the active device
        const player = window.spotifyPlayer;
        if (player) {
            const state = await player.getCurrentState();
            if (state) {
                // We're the active device, no need to make API call
                console.log('Using SDK state for active device');
                return;
            }
        }

        // Only make API call if we're not the active device
        //console.log('Falling back to API for remote device state');
        lastUpdateTime = Date.now();
        await wasm.get_current_playback();
        lastUpdateSuccess = true;
        
        // Reset tick timer whenever we get a server update
        lastTickTime = Date.now();
    } catch (error) {
        console.error('Error updating playback state:', error);
        lastUpdateSuccess = false;
    }
}

// Add at the top of the file after imports
let isInitializing = false;

function initializePlayer() {
    if (isInitializing) {
        console.log('Player initialization already in progress, skipping...');
        return;
    }

    const token = localStorage.getItem('spotify_token');
    if (!token) {
        console.warn('No token available for player initialization');
        set_sdk_status('No Token');
        return;
    }

    isInitializing = true;
    console.log('Starting player initialization...');

    // Cleanup existing player more thoroughly
    if (window.spotifyPlayer) {
        console.log('Cleaning up existing player...');
        window.spotifyPlayer.disconnect();
        window.spotifyPlayer = null;
    }

    // Get custom player name from localStorage or use default
    const playerName = localStorage.getItem('player_name') || 'Web Playback SDK Quick Start Player';
    console.log('Creating new player instance with name:', playerName);

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
        if (state) {
            // Update all window state from SDK state
            window.currentPlayerState = state;
            window.currentPlaybackTime = state.position;
            window.totalDuration = state.duration;
            window.isPlaying = !state.paused;
            window.shuffleState = state.shuffle;
            // Reset the update timer since we just got fresh state
            lastUpdateTime = Date.now();
            lastTickTime = Date.now(); // Reset tick timer on state change
            lastUpdateSuccess = true;
        }
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
            isInitializing = false;  // Reset flag regardless of outcome
            if (success) {
                console.log('Successfully connected to Spotify Player');
                set_sdk_status('Connected');
                window.spotifyPlayer = player;
                
                // Wait a short moment before starting updates
                await new Promise(resolve => setTimeout(resolve, 500));
                startPlaybackUpdates();

                // Add seek functionality
                window.seekTo = async (position_ms) => {
                    console.log('seekTo called:', position_ms);
                    const state = await player.getCurrentState();
                    
                    if (state) {
                        // Use SDK for local device
                        await player.seek(position_ms);
                        console.log('Seek completed via SDK');
                    } else {
                        // Use API for remote device
                        console.log('Using API for remote device seek');
                        await wasm.seek_playback(position_ms);
                    }
                };
                
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
                        // Use SDK for local device
                        if (state.paused) {
                            await player.resume();
                            console.log('Playback resumed via SDK');
                            set_sdk_status('Playing');
                        } else {
                            await player.pause();
                            console.log('Playback paused via SDK');
                            set_sdk_status('Paused');
                        }
                    } else {
                        // Use API for remote device
                        console.log('Using API for remote device control');
                        const isPlaying = window.isPlaying;
                        if (isPlaying) {
                            await wasm.pause_playback();
                            console.log('Playback paused via API');
                            set_sdk_status('Paused');
                        } else {
                            await wasm.resume_playback();
                            console.log('Playback resumed via API');
                            set_sdk_status('Playing');
                        }
                    }
                };
            } else {
                console.log('Failed to connect to Spotify Player');
                set_sdk_status('Connection Failed');
            }
        })
        .catch(error => {
            isInitializing = false;  // Reset flag on error
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
