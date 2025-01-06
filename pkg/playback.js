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
    // Store current state for UI access
    let currentState = null;
    player.addListener('player_state_changed', state => { 
        console.log('Player state changed:', state);
        currentState = state;
        window.currentPlayerState = state;
    });

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
                
                // Add play/pause functionality
                // Store device ID and enable play button when ready
                let deviceId;
                let isReady = false;
                
                player.addListener('ready', ({ device_id }) => {
                    console.log('Ready with Device ID', device_id);
                    deviceId = device_id;
                    isReady = true;
                    set_sdk_status('Ready');
                });

                window.playPause = async () => {
                    if (!isReady) {
                        console.error('Player not ready yet');
                        set_sdk_status('Not Ready');
                        return;
                    }
                    console.log('playPause called');
                    const state = await player.getCurrentState();
                    console.log('Current player state:', state);
                    
                    if (!state) {
                        // Start playback of a default track if nothing is playing
                        console.log('Starting default playback');
                        const token = localStorage.getItem('spotify_token');
                        
                        try {
                            const response = await fetch(`https://api.spotify.com/v1/me/player/play?device_id=${deviceId}`, {
                                method: 'PUT',
                                headers: {
                                    'Authorization': `Bearer ${token}`,
                                    'Content-Type': 'application/json'
                                },
                                body: JSON.stringify({
                                    uris: ['spotify:track:4uLU6hMCjMI75M1A2tKUQC'] // Default track URI
                                })
                            });
                            
                            if (response.ok) {
                                console.log('Started default playback');
                                set_sdk_status('Playing');
                            } else {
                                console.error('Failed to start playback');
                                set_sdk_status('Playback Error');
                            }
                        } catch (error) {
                            console.error('Error starting playback:', error);
                            set_sdk_status('Playback Error');
                        }
                    } else if (state.paused) {
                        await player.resume();
                        console.log('Playback resumed');
                        set_sdk_status('Playing');
                    } else {
                        await player.pause();
                        console.log('Playback paused');
                        set_sdk_status('Paused');
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

// Export for use in other modules
export { initializePlayer };
