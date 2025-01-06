import { set_sdk_status } from './spotify_egui.js';

console.log('playback.js loaded'); // Added initial load log

// Load the Spotify Web Playback SDK
window.onSpotifyWebPlaybackSDKReady = () => {
    console.log('Spotify Web Playback SDK is ready'); // Log SDK readiness
    const token = localStorage.getItem('spotify_token');
    const player = new Spotify.Player({
        name: 'Web Playback SDK Quick Start Player',
        getOAuthToken: cb => { cb(token); }
    });

    // Error handling
    player.addListener('initialization_error', ({ message }) => { console.error(message); });
    player.addListener('authentication_error', ({ message }) => { console.error(message); });
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
            } else {
                console.log('Failed to connect to Spotify Player');
            }
        })
        .catch(error => {
            console.error('Error connecting to Spotify Player:', error);
        });
};
