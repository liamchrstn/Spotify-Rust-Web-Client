import { set_sdk_status } from './spotify_egui.js';

console.log('playback.js loaded'); // Added initial load log

// Define the callback before loading the SDK
window.onSpotifyWebPlaybackSDKReady = () => {
    console.log('Spotify Web Playback SDK is ready');
    initializePlayer();
};

async function getCurrentPlayback() {
    const token = localStorage.getItem('spotify_token');
    if (!token) return null;

    try {
        const response = await fetch('https://api.spotify.com/v1/me/player/currently-playing', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });

        if (response.status === 204) {
            return null; // No track playing
        }

        if (response.ok) {
            const data = await response.json();
            return {
                track_window: {
                    current_track: {
                        name: data.item.name,
                        album: {
                            images: data.item.album.images
                        },
                        artists: data.item.artists
                    }
                },
                paused: !data.is_playing,
                position: data.progress_ms,
                duration: data.item.duration_ms
            };
        }
    } catch (error) {
        console.error('Error fetching current playback:', error);
        return null;
    }
}

function startPlaybackUpdates() {
    // Update immediately
    updatePlaybackState();
    // Then update every 200ms instead of 1000ms
    setInterval(updatePlaybackState, 200);
}

async function updatePlaybackState() {
    const state = await getCurrentPlayback();
    if (state) {
        window.currentPlayerState = state;
        window.currentPlaybackTime = state.position;
        window.totalDuration = state.duration;
        window.isPlaying = !state.paused;
    }
}

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
                startPlaybackUpdates(); // Add this line
                
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
                        // Check queue first
                        const token = localStorage.getItem('spotify_token');
                        try {
                            // Check queue
                            const queueResponse = await fetch('https://api.spotify.com/v1/me/player/queue', {
                                headers: {
                                    'Authorization': `Bearer ${token}`
                                }
                            });
                            
                            if (queueResponse.ok) {
                                const queueData = await queueResponse.json();
                                if (queueData.queue && queueData.queue.length > 0) {
                                    // Play next song in queue
                                    await fetch(`https://api.spotify.com/v1/me/player/play?device_id=${deviceId}`, {
                                        method: 'PUT',
                                        headers: {
                                            'Authorization': `Bearer ${token}`
                                        }
                                    });
                                } else {
                                    // If queue is empty, play from saved tracks
                                    const savedTracksResponse = await fetch('https://api.spotify.com/v1/me/tracks?limit=50', {
                                        headers: {
                                            'Authorization': `Bearer ${token}`
                                        }
                                    });
                                    
                                    if (savedTracksResponse.ok) {
                                        const savedTracks = await savedTracksResponse.json();
                                        if (savedTracks.items && savedTracks.items.length > 0) {
                                            // Enable shuffle before playing
                                            await fetch(`https://api.spotify.com/v1/me/player/shuffle?state=true&device_id=${deviceId}`, {
                                                method: 'PUT',
                                                headers: {
                                                    'Authorization': `Bearer ${token}`
                                                }
                                            });
                                            
                                            const trackUris = savedTracks.items.map(item => item.track.uri);
                                            await fetch(`https://api.spotify.com/v1/me/player/play?device_id=${deviceId}`, {
                                                method: 'PUT',
                                                headers: {
                                                    'Authorization': `Bearer ${token}`,
                                                    'Content-Type': 'application/json'
                                                },
                                                body: JSON.stringify({
                                                    uris: trackUris
                                                })
                                            });
                                        }
                                    }
                                }
                            }
                            set_sdk_status('Playing');
                        } catch (error) {
                            console.error('Error managing playback:', error);
                            set_sdk_status('Playback Error');
                        }
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
