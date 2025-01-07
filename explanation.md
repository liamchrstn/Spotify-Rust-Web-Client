# Pre-WASM Implementation Documentation

## index.html

The index.html file serves as the entry point for the web application, providing the basic HTML structure and loading all necessary resources. Here's a detailed technical breakdown:

### Structure Overview
- Uses HTML5 doctype with English language specification
- Implements responsive design through viewport meta tag
- Contains both the application interface and loading elements

### Head Section
```html
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust Spotify</title>
    <link rel="icon" href="favicon.png" type="image/png">
    <link rel="stylesheet" href="main.css">
</head>
```
- Character encoding set to UTF-8 for universal character support
- Viewport meta tag ensures proper scaling on mobile devices
- Custom favicon integration for brand identity
- External CSS stylesheet linked for styling (main.css)

### Body Section
```html
<body>
    <script type="module" src="spotify_egui.js"></script>
    <div class="pl">
        <div class="pl__bar"></div>
        <div class="pl__bar"></div>
        <div class="pl__bar"></div>
    </div>
    <canvas id="canvas" style="display: none;"></canvas>
</body>
```

#### Key Components:
1. **Loading Animation**
   - Custom loading animation implemented through `pl` class
   - Uses three bar elements (`pl__bar`) for visual feedback
   - Provides user feedback during application initialization

2. **Canvas Element**
   - Hidden by default (`display: none`)
   - Used by WASM/egui for rendering the application interface
   - Identified by 'canvas' ID for JavaScript reference

### Script Loading
```html
<script type="module" src="spotify_egui.js"></script>
<script type="module" src="playback.js"></script>
<script type="module" src="auth.js"></script>
<script type="module" src="api.js"></script>
<script src="https://sdk.scdn.co/spotify-player.js"></script>
```

#### Script Loading Order and Purpose:
1. **spotify_egui.js**
   - Primary WASM interface module
   - Handles communication between Rust and JavaScript

2. **playback.js**
   - Manages music playback functionality
   - Controls player state and interactions

3. **auth.js**
   - Handles Spotify authentication
   - Manages OAuth tokens and authorization flow

4. **api.js**
   - Implements Spotify API interactions
   - Handles data fetching and API requests

5. **spotify-player.js**
   - External Spotify Web Playback SDK
   - Enables direct playback of Spotify content

### Technical Notes
- Uses ES6 modules (type="module") for better code organization and scoping
- Implements progressive loading strategy for better performance
- Canvas element serves as the primary interface between WASM and DOM
- Loading animation provides visual feedback during application initialization

## auth.js

The auth.js file implements Spotify's OAuth 2.0 authentication flow using PKCE (Proof Key for Code Exchange), providing secure user authentication and token management. Here's a detailed technical breakdown:

### Configuration Constants
```javascript
const clientId = '75a6782d877a45d9adf93299e1663ad9';
const redirectUri = 'http://localhost:8000/';
const authEndpoint = 'https://accounts.spotify.com/authorize';
const tokenEndpoint = 'https://accounts.spotify.com/api/token';
```
- Defines essential OAuth configuration parameters
- Uses PKCE flow for enhanced security
- Implements comprehensive scope permissions for Spotify API access

### Token Management System
```javascript
const tokenStorage = {
    setAccessToken: (token) => localStorage.setItem('spotify_token', token),
    getAccessToken: () => localStorage.getItem('spotify_token'),
    setRefreshToken: (token) => localStorage.setItem('spotify_refresh_token', token),
    getRefreshToken: () => localStorage.getItem('spotify_refresh_token'),
    setExpiryTime: (time) => localStorage.setItem('spotify_token_expiry', time),
    getExpiryTime: () => localStorage.getItem('spotify_token_expiry'),
    clear: () => { ... }
};
```
- Implements persistent token storage using localStorage
- Manages access tokens, refresh tokens, and expiry times
- Provides methods for token lifecycle management

### PKCE Implementation
1. **Code Verifier Generation**
```javascript
function generateRandomString(length) {
    const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    const values = crypto.getRandomValues(new Uint8Array(length));
    return values.reduce((acc, x) => acc + possible[x % possible.length], '');
}
```
- Uses cryptographically secure random values
- Generates a random string for PKCE verification
- Implements RFC-compliant character set

2. **Code Challenge Generation**
```javascript
async function generateCodeChallenge(codeVerifier) {
    const encoder = new TextEncoder();
    const data = encoder.encode(codeVerifier);
    const digest = await crypto.subtle.digest('SHA-256', data);
    return btoa(String.fromCharCode(...new Uint8Array(digest)))
        .replace(/=/g, '')
        .replace(/\+/g, '-')
        .replace(/\//g, '_');
}
```
- Implements SHA-256 hashing for code challenge
- Uses Web Crypto API for secure hash generation
- Applies base64URL encoding with proper character substitution

### Authentication Flow Functions
1. **Login Initiation**
```javascript
export async function loginWithSpotify() {
    const codeVerifier = generateRandomString(64);
    const codeChallenge = await generateCodeChallenge(codeVerifier);
    // ... authorization URL construction and redirect
}
```
- Initiates OAuth flow with PKCE
- Stores code verifier for later verification
- Constructs and redirects to Spotify authorization URL

2. **Token Exchange**
```javascript
async function exchangeToken(code) {
    // ... token exchange implementation
}
```
- Handles authorization code exchange for access token
- Implements error handling for failed exchanges
- Stores received tokens and expiry times

3. **Token Refresh**
```javascript
async function refreshAccessToken() {
    // ... refresh token implementation
}
```
- Automatically refreshes expired access tokens
- Implements error handling and recovery
- Updates stored tokens upon successful refresh

### Token Access Management
```javascript
export async function getAccessToken() {
    // ... token management logic
}
```
- Provides centralized token access
- Handles automatic token refresh when needed
- Implements URL parameter parsing for authorization code

### Technical Notes
- Implements full OAuth 2.0 PKCE flow for security
- Uses modern Web Crypto API for secure operations
- Provides automatic token refresh mechanism
- Implements proper error handling and recovery
- Uses localStorage for persistent token storage
- Exports necessary functions for external module use
- Attaches login function to window for global access

## api.js

Despite its name, api.js serves as the main application initialization module, orchestrating the startup sequence and coordination between WASM, authentication, and playback components. Here's a detailed technical breakdown:

### Module Imports
```javascript
import init, { start, set_access_token } from './spotify_egui.js';
import { getAccessToken } from './auth.js';
import { initializePlayer } from './playback.js';
```
- Imports WASM initialization and control functions
- Imports authentication utilities
- Imports player initialization function

### Application Initialization
```javascript
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
```

#### Initialization Sequence
1. **WASM Module Initialization**
   - Calls `init()` to load and initialize the WASM module
   - Ensures WASM components are ready before proceeding

2. **Authentication Setup**
   - Retrieves access token using auth.js utilities
   - Sets token in WASM module if available
   - Enables authenticated API calls

3. **Player Initialization**
   - Checks for Spotify SDK availability
   - Initializes player if SDK is loaded
   - Ensures playback functionality is ready

4. **UI Transition**
   - Starts the WASM application
   - Manages loading animation visibility
   - Shows the application canvas

### Event Handling
```javascript
window.addEventListener('load', run);
```
- Triggers initialization on page load
- Ensures DOM is fully loaded before starting
- Provides clean startup sequence

### Error Handling
- Implements comprehensive error catching
- Logs initialization errors to console
- Ensures loading animation is hidden on error
- Prevents application from hanging on failure

### Technical Notes
- Uses ES6 module system for dependency management
- Implements async/await for proper initialization sequencing
- Manages UI state transitions during startup
- Coordinates between multiple system components:
  * WASM module
  * Authentication system
  * Playback system
  * UI elements
- Provides graceful error handling and recovery
- Ensures proper resource initialization order

## playback.js

The playback.js file implements the Spotify Web Playback SDK integration, providing playback control and state management for the Spotify player. Here's a detailed technical breakdown:

### SDK Integration
```javascript
import { set_sdk_status } from './spotify_egui.js';

window.onSpotifyWebPlaybackSDKReady = () => {
    console.log('Spotify Web Playback SDK is ready');
    initializePlayer();
};
```
- Imports SDK status management from WASM interface
- Implements SDK ready callback
- Triggers player initialization when SDK loads

### Player Initialization
```javascript
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
}
```
- Retrieves authentication token
- Configures Spotify Web Playback SDK
- Implements token callback mechanism

### Event Listeners
1. **Error Handling**
```javascript
player.addListener('initialization_error', ({ message }) => {
    console.error(message);
    set_sdk_status('Init Error');
});
player.addListener('authentication_error', ({ message }) => {
    console.error(message);
    set_sdk_status('Auth Error');
});
```
- Handles various error scenarios:
  * Initialization errors
  * Authentication failures
  * Account errors
  * Playback issues
- Updates SDK status for UI feedback

2. **State Management**
```javascript
let currentState = null;
player.addListener('player_state_changed', state => { 
    console.log('Player state changed:', state);
    currentState = state;
    window.currentPlayerState = state;
});
```
- Tracks player state changes
- Maintains current state reference
- Exposes state to window object for debugging

3. **Device Management**
```javascript
player.addListener('ready', ({ device_id }) => {
    console.log('Ready with Device ID', device_id);
    set_sdk_status('Ready');
});

player.addListener('not_ready', ({ device_id }) => {
    console.log('Device ID has gone offline', device_id);
    set_sdk_status('Not Ready');
});
```
- Handles device ready/not ready states
- Manages device ID for playback control
- Updates SDK status accordingly

### Playback Control
```javascript
window.playPause = async () => {
    if (!isReady) {
        console.error('Player not ready yet');
        set_sdk_status('Not Ready');
        return;
    }
    // ... playback control implementation
};
```
- Implements play/pause functionality
- Handles default track playback
- Manages player state transitions
- Provides error handling for playback operations

### Player Connection
```javascript
player.connect()
    .then(success => {
        if (success) {
            console.log('Successfully connected to Spotify Player');
            set_sdk_status('Connected');
            // ... additional setup
        }
    })
    .catch(error => {
        console.error('Error connecting to Spotify Player:', error);
        set_sdk_status('Connection Error');
    });
```
- Establishes connection to Spotify servers
- Handles connection success/failure
- Sets up playback controls on successful connection
- Implements error handling for connection issues

### Technical Notes
- Implements comprehensive error handling
- Provides detailed logging for debugging
- Manages player state transitions
- Handles device connectivity
- Implements playback control interface
- Coordinates with WASM module through status updates
- Uses ES6 module system for dependency management
- Provides global access to playback controls
- Implements automatic state recovery
- Handles token-based authentication
- Manages device ID for multi-device scenarios
- Implements default track fallback

## main.css

The main.css file implements the application's styling and animations, with a particular focus on the loading animation and responsive design. Here's a detailed technical breakdown:

### Base Styles and Reset
```css
body { margin: 0; }
canvas { width: 100vw; height: 100vh; }
* {
    border: 0;
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}
```
- Implements CSS reset for consistent rendering
- Sets full viewport canvas dimensions
- Uses border-box model for predictable layouts

### CSS Variables and Theming
```css
:root {
    --hue: 223;
    --bg: #1b1b1b;
    --fg: hsl(var(--hue),90%,10%);
    --primary: hsl(var(--hue),90%,60%);
    --secondary: hsl(283,90%,60%);
    --trans-dur: 0.3s;
    --bezier: cubic-bezier(0.65,0,0.35,1);
    font-size: calc(14px + (30 - 14) * (100vw - 280px) / (3840 - 280));
}
```
- Defines global CSS custom properties
- Implements responsive font scaling
- Uses HSL color model for flexible theming
- Defines animation timing variables
- Implements custom easing function

### Body Styling
```css
body {
    background-color: var(--bg);
    color: var(--fg);
    display: flex;
    font: 1em/1.5 sans-serif;
    height: 100vh;
    transition:
        background-color var(--trans-dur),
        color var(--trans-dur);
}
```
- Sets dark theme background
- Implements flexible layout with flexbox
- Adds smooth color transitions
- Uses viewport-relative height

### Loading Animation
1. **Container Configuration**
```css
.pl {
    --dur: 5s;
    --size: 8em;
    --bar-width: calc(var(--size) * 0.25);
    aspect-ratio: 1 / 1;
    display: flex;
    justify-content: space-between;
    margin: auto;
    width: var(--size);
}
```
- Defines animation duration
- Sets responsive dimensions
- Uses CSS calculations for proportions
- Implements flexbox layout

2. **Bar Elements**
```css
.pl__bar {
    background-color: var(--primary);
    position: relative;
    width: var(--bar-width);
    height: 100%;
    transform-style: preserve-3d;
    animation:
        bar-color var(--dur) step-end infinite,
        bar-spin var(--dur) var(--bezier) infinite;
}
```
- Implements 3D transforms
- Uses multiple animations
- Applies custom timing function
- Sets infinite animation loop

3. **Pseudo-elements**
```css
.pl__bar:before,
.pl__bar:after {
    animation-timing-function: step-end;
    background-color: var(--fg);
    content: "";
    display: block;
    position: absolute;
    /* ... positioning and dimensions */
}
```
- Creates additional animation elements
- Uses pseudo-elements for efficiency
- Implements complex 3D positioning
- Adds smooth color transitions

### Animation Keyframes
```css
@keyframes bar-color {
    /* Color transition keyframes */
}
@keyframes bar-spin {
    /* 3D rotation keyframes */
}
@keyframes bar-end-1 {
    /* End cap animation 1 */
}
@keyframes bar-end-2 {
    /* End cap animation 2 */
}
```
- Implements color transitions
- Defines 3D rotations
- Manages border radius changes
- Coordinates multiple animations

### Dark Mode Support
```css
@media (prefers-color-scheme: dark) {
    :root {
        --bg: #1b1b1b;
        --fg: hsl(var(--hue),90%,90%);
    }
}
```
- Implements system dark mode detection
- Adjusts colors for dark theme
- Maintains consistent contrast ratios
- Uses CSS custom properties for theming

### Technical Notes
- Uses modern CSS features:
  * Custom properties
  * Calc() functions
  * 3D transforms
  * Flexbox layout
  * Aspect ratio
  * Media queries
- Implements responsive design principles
- Provides smooth animations and transitions
- Supports system color scheme preferences
- Uses efficient animation techniques
- Implements maintainable theming system
- Ensures cross-browser compatibility
- Optimizes performance with hardware acceleration

## spotify_egui.js

The spotify_egui.js file serves as the WebAssembly binding layer between JavaScript and Rust, providing the interface for communication between the web frontend and the Rust backend. Here's a detailed technical breakdown:

### Memory Management
```javascript
const heap = new Array(128).fill(undefined);
heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }
function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];
    heap[idx] = obj;
    return idx;
}
```
- Implements custom heap management
- Handles JavaScript object references
- Provides efficient memory allocation
- Manages object lifecycle

### Text Encoding/Decoding
```javascript
const cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
const cachedTextEncoder = new TextEncoder('utf-8');
```
- Implements UTF-8 text encoding/decoding
- Uses cached encoder/decoder instances
- Handles binary-to-string conversions
- Manages WebAssembly string operations

### WebAssembly Interface
```javascript
export function set_access_token(token) {
    const ptr0 = passStringToWasm0(token, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.set_access_token(ptr0, len0);
}

export function set_sdk_status(status) {
    const ptr0 = passStringToWasm0(status, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.set_sdk_status(ptr0, len0);
}
```
- Exports functions for Rust interaction
- Handles token management
- Manages SDK status updates
- Provides WebAssembly memory management

### Error Handling
```javascript
function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}
```
- Implements comprehensive error handling
- Manages JavaScript-to-Rust error propagation
- Provides error context preservation
- Ensures proper error recovery

### WebAssembly Initialization
```javascript
async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;
    const imports = __wbg_get_imports();
    if (typeof module_or_path === 'string' || module_or_path instanceof Request || module_or_path instanceof URL) {
        module_or_path = fetch(module_or_path);
    }
    const { instance, module } = await __wbg_load(await module_or_path, imports);
    return __wbg_finalize_init(instance, module);
}
```
- Manages WebAssembly module initialization
- Handles dynamic module loading
- Implements import binding
- Provides initialization finalization

### DOM Integration
```javascript
imports.wbg.__wbg_instanceof_Window_d2514c6a7ee7ba60 = function(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};
```
- Provides DOM type checking
- Implements instance verification
- Handles browser API integration
- Manages DOM event binding

### WebGL Context
```javascript
imports.wbg.__wbg_getContext_5eaf5645cd6acb46 = function() {
    return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    }, arguments);
};
```
- Manages WebGL context creation
- Handles graphics initialization
- Provides context error handling
- Implements context management

### Technical Notes
- Implements WebAssembly binding layer
- Provides memory management utilities
- Handles string encoding/decoding
- Manages object references
- Implements error propagation
- Provides DOM integration
- Handles WebGL context
- Uses ES6 module system
- Implements async initialization
- Provides type checking utilities
- Manages resource cleanup
- Implements secure memory handling
- Provides debugging capabilities
- Ensures cross-platform compatibility

## package.json

The package.json file defines the npm package configuration for the WebAssembly module and its JavaScript bindings. Here's a detailed technical breakdown:

### Package Metadata
```json
{
  "name": "spotify-egui",
  "version": "0.1.0",
  "type": "module"
}
```
- Defines package name and version
- Specifies ES module type for modern JavaScript
- Follows semantic versioning

### File Distribution
```json
{
  "files": [
    "spotify_egui_bg.wasm",
    "spotify_egui.js",
    "spotify_egui.d.ts"
  ]
}
```
- Lists distributed package files:
  * WebAssembly binary (`.wasm`)
  * JavaScript bindings (`.js`)
  * TypeScript definitions (`.d.ts`)
- Ensures proper package bundling

### Entry Points
```json
{
  "main": "spotify_egui.js",
  "types": "spotify_egui.d.ts"
}
```
- Specifies main JavaScript entry point
- Provides TypeScript type definitions
- Enables proper module resolution

### Side Effects
```json
{
  "sideEffects": [
    "./snippets/*"
  ]
}
```
- Declares side effect patterns
- Optimizes tree-shaking
- Improves bundler optimization

### Technical Notes
- Implements npm package configuration
- Supports WebAssembly distribution
- Provides TypeScript integration
- Enables ES module usage
- Optimizes bundle size
- Ensures proper module resolution
- Maintains package integrity
- Supports modern build tools
- Enables efficient dependency management
