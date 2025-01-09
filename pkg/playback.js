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
            // Add null check for data.item
            if (!data || !data.item) {
                console.log('No current track data available');
                return null;
            }
            
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

    // Add shuffle state to the update
    const fetchShuffleState = async () => {
        const response = await fetch('https://api.spotify.com/v1/me/player', {
            headers: {
                'Authorization': `Bearer ${localStorage.getItem('spotify_token')}`
            }
        });
        if (response.ok) {
            const data = await response.json();
            window.shuffleState = data.shuffle_state;
        }
    };
    fetchShuffleState();
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

async function skipToNext() {
    const token = localStorage.getItem('spotify_token');
    if (!token) return;

    try {
        await fetch('https://api.spotify.com/v1/me/player/next', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
    } catch (error) {
        console.error('Error skipping to next track:', error);
    }
}

async function skipToPrevious() {
    const token = localStorage.getItem('spotify_token');
    if (!token) return;

    try {
        await fetch('https://api.spotify.com/v1/me/player/previous', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
    } catch (error) {
        console.error('Error skipping to previous track:', error);
    }
}

async function toggleShuffle() {
    const token = localStorage.getItem('spotify_token');
    if (!token) return;

    try {
        // Get current state first
        const response = await fetch('https://api.spotify.com/v1/me/player', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        if (response.ok) {
            const data = await response.json();
            const newState = !data.shuffle_state;
            
            await fetch(`https://api.spotify.com/v1/me/player/shuffle?state=${newState}`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });
            
            // Update shuffle state in window
            window.shuffleState = newState;
        }
    } catch (error) {
        console.error('Error toggling shuffle:', error);
    }
}

async function getDevices() {
    const token = localStorage.getItem('spotify_token');
    if (!token) return [];

    try {
        const response = await fetch('https://api.spotify.com/v1/me/player/devices', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        if (response.ok) {
            const data = await response.json();
            window.availableDevices = data.devices;
            return data.devices;
        }
    } catch (error) {
        console.error('Error fetching devices:', error);
    }
    return [];
}

async function transferPlayback(deviceId) {
    const token = localStorage.getItem('spotify_token');
    if (!token) return;

    try {
        await fetch('https://api.spotify.com/v1/me/player', {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                device_ids: [deviceId],
                play: true
            })
        });
    } catch (error) {
        console.error('Error transferring playback:', error);
    }
}

async function activateDevice(deviceId) {
    const token = localStorage.getItem('spotify_token');
    if (!token || !deviceId) return false;

    try {
        console.log('Activating device:', deviceId);
        const response = await fetch('https://api.spotify.com/v1/me/player', {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                device_ids: [deviceId],
                play: false
            })
        });
        
        // Wait for device activation
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Verify device is active
        const verifyResponse = await fetch('https://api.spotify.com/v1/me/player', {
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        if (verifyResponse.ok) {
            const data = await verifyResponse.json();
            return data?.device?.id === deviceId;
        }
        return false;
    } catch (error) {
        console.error('Error activating device:', error);
        return false;
    }
}

async function hasActiveDevices() {
    const devices = await getDevices();
    return devices.some(device => device.is_active);
}

async function initializePlayback(fromPlayMusic = false) {
    const token = localStorage.getItem('spotify_token');
    if (!token) return;

    try {
        // Only do basic device activation on initialization
        const deviceId = window.deviceId;
        if (deviceId) {
            await activateDevice(deviceId);
        }
    } catch (error) {
        console.error('Error initializing playback:', error);
        set_sdk_status('Playback Error: ' + error.message);
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
        window.deviceId = device_id;  // Store device ID globally
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
        .then(async success => {
            if (success) {
                console.log('Successfully connected to Spotify Player');
                set_sdk_status('Connected');
                window.isReady = true;  // Set ready status first
                window.spotifyPlayer = player;
                
                // Wait a short moment before starting updates and initialization
                await new Promise(resolve => setTimeout(resolve, 500));
                
                startPlaybackUpdates();
                initializePlayback();
                
                // Rest of the connect success code...
                // Make the player globally accessible for seeking
                window.spotifyPlayer = player;
                
                // Initialize playback when player is ready
                initializePlayback();

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
                        // Check if there are no active devices before attempting transfer
                        const noActiveDevices = !(await hasActiveDevices());
                        if (noActiveDevices) {
                            console.log('No active devices found, attempting playback transfer...');
                            const transferred = await attemptPlaybackTransfer();
                            if (transferred) {
                                console.log('Playback transferred successfully');
                                set_sdk_status('Playing');
                                return;
                            }
                        }
                        console.log('No playback state available');
                        set_sdk_status('No Playback');
                    }
                };

                // Add skip functions to window
                window.skipToNext = skipToNext;
                window.skipToPrevious = skipToPrevious;
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

// Add to window object
window.toggleShuffle = toggleShuffle;
window.getDevices = getDevices;
window.transferPlayback = transferPlayback;

// Export for use in other modules
export { initializePlayer };

async function attemptPlaybackTransfer() {
    const deviceId = window.deviceId;
    if (!deviceId) return false;

    console.log('Attempting playback transfer...');
    const isActive = await activateDevice(deviceId);
    if (!isActive) {
        console.log('Failed to activate device');
        return false;
    }

    const token = localStorage.getItem('spotify_token');
    if (!token) return false;

    try {
        // Try to start playback from saved tracks
        const savedTracksResponse = await fetch('https://api.spotify.com/v1/me/tracks?limit=50', {
            headers: { 'Authorization': `Bearer ${token}` }
        });

        if (savedTracksResponse.ok) {
            const savedTracks = await savedTracksResponse.json();
            if (savedTracks.items?.length > 0) {
                const trackUris = savedTracks.items.map(item => item.track.uri);
                await fetch('https://api.spotify.com/v1/me/player/play', {
                    method: 'PUT',
                    headers: {
                        'Authorization': `Bearer ${token}`,
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({
                        uris: trackUris,
                        device_id: deviceId
                    })
                });
                return true;
            }
        }
    } catch (error) {
        console.error('Error during playback transfer:', error);
    }
    return false;
}

// Update the playPause function
window.playPause = async () => {
    if (!window.isReady) {
        console.error('Player not ready yet');
        set_sdk_status('Not Ready');
        return;
    }
    
    console.log('playPause called');
    const state = await window.spotifyPlayer.getCurrentState();
    console.log('Current player state:', state);
    
    if (!state) {
        // Check if there are no active devices before attempting transfer
        const noActiveDevices = !(await hasActiveDevices());
        if (noActiveDevices) {
            console.log('No active devices found, attempting playback transfer...');
            const transferred = await attemptPlaybackTransfer();
            if (transferred) {
                console.log('Playback transferred successfully');
                set_sdk_status('Playing');
                return;
            }
        }
        console.log('No playback state available');
        set_sdk_status('No Playback');
        return;
    }
    
    // Normal play/pause logic for when we have a state
    if (state.paused) {
        await window.spotifyPlayer.resume();
        console.log('Playback resumed');
        set_sdk_status('Playing');
    } else {
        await window.spotifyPlayer.pause();
        console.log('Playback paused');
        set_sdk_status('Paused');
    }
};
